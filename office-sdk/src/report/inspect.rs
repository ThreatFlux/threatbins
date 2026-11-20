use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
    path::{Component, Path, PathBuf},
};

/// Verify returned bytes against an SDK manifest and inspect structural parts.
/// This does not replace visual review or establish application approval.
pub fn inspect_report(bytes: &[u8], manifest: &ReportManifest) -> ReportResult<ReportInspection> {
    if bytes.len() != manifest.byte_size || sha256(bytes) != manifest.output_sha256 {
        return Err(ReportError::DigestMismatch);
    }
    if bytes.len() > ReportLimits::default().max_output_bytes {
        return Err(ReportError::LimitExceeded("inspection bytes".into()));
    }
    let template = Template::parse(&manifest.template)?;
    if manifest.schema_version != REPORT_SCHEMA_VERSION
        || manifest.mime_type != manifest.format.mime_type()
        || (manifest.format == ReportFormat::Html && manifest.pages_or_slides.is_some())
    {
        return Err(invalid(
            "manifest",
            "unsupported schema/template or MIME mismatch",
        ));
    }
    let binding = (template == Template::SectionFirst)
        .then(|| manifest.artifact_binding())
        .transpose()?;
    let mut result = manifest.inspection.clone();
    result.native_editable_text = false;
    result.native_editable_tables = false;
    result.native_editable_charts = false;
    result.structural_checks.clear();
    match manifest.format {
        ReportFormat::Pdf => {
            let pdf =
                lopdf::Document::load_mem(bytes).map_err(|_| invalid("pdf", "unreadable PDF"))?;
            if Some(pdf.get_pages().len()) != manifest.pages_or_slides {
                return Err(invalid("pdf", "page count mismatch"));
            }
            if let Ok(info) = pdf
                .trailer
                .get(b"Info")
                .and_then(lopdf::Object::as_reference)
                && let Ok(info) = pdf.get_dictionary(info)
                && let Ok(subject) = info.get(b"Subject").and_then(lopdf::Object::as_str)
            {
                let subject = pdf_text(subject);
                verify_binding(
                    subject.as_deref().filter(|s| s.starts_with(BINDING_PREFIX)),
                    binding.as_deref(),
                )?;
            } else {
                verify_binding(None, binding.as_deref())?;
            }
            result
                .structural_checks
                .push("PDF parsed and page count matches manifest".into());
        }
        ReportFormat::Html => {
            let h = std::str::from_utf8(bytes).map_err(|_| invalid("html", "not UTF-8"))?;
            inspect_html(h, &manifest.document_sha256, binding.as_deref())?;
            result
                .structural_checks
                .push("UTF-8 offline HTML with document binding and no active content".into());
        }
        ReportFormat::Pptx => {
            let mut expanded = 0u64;
            let mut part_count = 0usize;
            let parts = read_package(bytes, &mut expanded, &mut part_count)?;
            for (name, data) in &parts {
                if name.ends_with(".xml") || name.ends_with(".rels") {
                    parse_xml(data)?;
                }
                if name.ends_with(".rels") {
                    check_rels(name, data, &parts)?;
                }
            }
            check_theme_style_matrix(
                parts
                    .get("ppt/theme/theme1.xml")
                    .ok_or_else(|| invalid("pptx", "missing report theme"))?,
            )?;
            let keywords = parts
                .get("docProps/core.xml")
                .map(|core| core_keywords(core))
                .transpose()?
                .flatten();
            verify_binding(
                keywords
                    .as_deref()
                    .filter(|s| s.starts_with(BINDING_PREFIX)),
                binding.as_deref(),
            )?;
            let presentation = std::str::from_utf8(
                parts
                    .get("ppt/presentation.xml")
                    .ok_or_else(|| invalid("pptx", "missing presentation"))?,
            )
            .map_err(|_| invalid("pptx", "not UTF-8"))?;
            if !presentation.contains("cx=\"12192000\" cy=\"6858000\"") {
                return Err(invalid("pptx", "not 16:9 widescreen"));
            }
            let count = parts
                .keys()
                .filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml"))
                .count();
            if Some(count) != manifest.pages_or_slides {
                return Err(invalid("pptx", "slide count mismatch"));
            }
            let charts = parts
                .keys()
                .filter(|n| n.starts_with("ppt/charts/chart") && n.ends_with(".xml"))
                .count();
            let workbooks = parts
                .keys()
                .filter(|n| n.starts_with("ppt/embeddings/") && n.ends_with(".xlsx"))
                .count();
            if template == Template::SectionFirst {
                result.native_editable_tables =
                    inspect_section_deck(&parts, presentation.as_bytes(), count, charts)?;
                result.native_editable_text = true;
                result.structural_checks.push(
                    "ordered native slides, notes, chart/workbook and navigation references verified".into(),
                );
            } else {
                result.native_editable_text = parts.iter().any(|(n, v)| {
                    n.starts_with("ppt/slides/slide") && String::from_utf8_lossy(v).contains("<a:t")
                });
                result.native_editable_tables = parts.iter().any(|(n, v)| {
                    n.starts_with("ppt/slides/slide")
                        && String::from_utf8_lossy(v).contains("<a:tbl>")
                });
            }
            result.native_editable_charts = charts > 0;
            if charts != workbooks {
                return Err(invalid("pptx", "chart/workbook count mismatch"));
            }
            for (name, data) in &parts {
                if name.starts_with("ppt/embeddings/") && name.ends_with(".xlsx") {
                    let workbook = read_package(data, &mut expanded, &mut part_count)?;
                    if !workbook.contains_key("xl/worksheets/sheet1.xml") {
                        return Err(invalid("xlsx", "missing chart worksheet"));
                    }
                    for (part, content) in &workbook {
                        if part.ends_with(".xml") || part.ends_with(".rels") {
                            parse_xml(content)?;
                        }
                        if part.ends_with(".rels") {
                            check_rels(part, content, &workbook)?;
                        }
                    }
                }
            }
            result.structural_checks.extend([
                "OOXML parts parse and every internal relationship resolves".into(),
                "saved 16:9 dimensions and slide count verified".into(),
                "DrawingML theme style matrix minimum entries verified".into(),
                match template {
                    Template::Executive => "chart parts reference embedded editable workbook data",
                    Template::SectionFirst => {
                        "chart and workbook counts match and embedded worksheets parse"
                    }
                }
                .into(),
            ]);
        }
    }
    if binding.is_some() {
        result
            .structural_checks
            .push("embedded section-first artifact binding matches manifest".into());
    }
    result
        .structural_checks
        .push("byte size and SHA-256 match manifest".into());
    result.visual_review_required = true;
    Ok(result)
}

fn verify_binding(actual: Option<&str>, expected: Option<&str>) -> ReportResult<()> {
    if actual != expected {
        return Err(invalid("manifest", "embedded report binding mismatch"));
    }
    Ok(())
}

fn pdf_text(bytes: &[u8]) -> Option<String> {
    if let Some(be) = bytes.strip_prefix(&[0xfe, 0xff]) {
        let (pairs, remainder) = be.as_chunks::<2>();
        if !remainder.is_empty() {
            return None;
        }
        let units = pairs
            .iter()
            .copied()
            .map(u16::from_be_bytes)
            .collect::<Vec<_>>();
        String::from_utf16(&units).ok()
    } else {
        String::from_utf8(bytes.to_vec()).ok()
    }
}

fn core_keywords(data: &[u8]) -> ReportResult<Option<String>> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_reader(data);
    let mut keywords = None;
    loop {
        match reader.read_event().map_err(crate::OfficeError::from)? {
            Event::Start(e) if e.name().as_ref() == "cp:keywords" => {
                if keywords.is_some() {
                    return Err(invalid("pptx", "duplicate core keywords"));
                }
                let text = reader
                    .read_text(e.name())
                    .map_err(crate::OfficeError::from)?;
                keywords = Some(text.xml10_content().into_owned());
            }
            Event::Eof => return Ok(keywords),
            _ => {}
        }
    }
}

type Attributes = BTreeMap<String, String>;

fn xml_elements(data: &[u8], names: &[&str]) -> ReportResult<Vec<(String, Attributes)>> {
    use quick_xml::{events::Event, name::ResolveResult};
    let mut reader = quick_xml::NsReader::from_reader(data);
    let mut elements = Vec::new();
    loop {
        match reader.read_event().map_err(crate::OfficeError::from)? {
            Event::Start(e) | Event::Empty(e) => {
                let qualified = e.name();
                let name = qualified.as_ref();
                if names.contains(&name) {
                    let namespace = match name.split_once(':').map(|(prefix, _)| prefix) {
                        Some("p") => "http://schemas.openxmlformats.org/presentationml/2006/main",
                        Some("a") => "http://schemas.openxmlformats.org/drawingml/2006/main",
                        Some("c") => "http://schemas.openxmlformats.org/drawingml/2006/chart",
                        _ => "http://schemas.openxmlformats.org/package/2006/relationships",
                    };
                    let (actual, _) = reader.resolver().resolve_element(qualified);
                    if !matches!(actual, ResolveResult::Bound(uri) if uri.as_ref() == namespace) {
                        return Err(invalid("pptx", "unexpected OOXML element namespace"));
                    }
                    let mut attrs = Attributes::new();
                    for attr in e.attributes() {
                        let attr = attr.map_err(|_| invalid("pptx", "invalid attribute"))?;
                        let name = attr.key.as_ref();
                        if name == "r:id" {
                            let (actual, _) = reader.resolver().resolve_attribute(attr.key);
                            if !matches!(actual, ResolveResult::Bound(uri) if uri.as_ref() == pptx::REL)
                            {
                                return Err(invalid(
                                    "pptx",
                                    "unexpected OOXML relationship namespace",
                                ));
                            }
                        }
                        let value = attr
                            .normalized_value(quick_xml::XmlVersion::Explicit1_0)
                            .map_err(crate::OfficeError::from)?;
                        attrs.insert(name.into(), value.into_owned());
                    }
                    elements.push((name.into(), attrs));
                }
            }
            Event::Eof => return Ok(elements),
            _ => {}
        }
    }
}

fn part<'a>(parts: &'a BTreeMap<String, Vec<u8>>, path: &str) -> ReportResult<&'a [u8]> {
    parts
        .get(path)
        .map(Vec::as_slice)
        .ok_or_else(|| invalid("pptx", &format!("missing {path}")))
}

fn relationships(
    parts: &BTreeMap<String, Vec<u8>>,
    path: &str,
) -> ReportResult<BTreeMap<String, (String, String)>> {
    let mut relations = BTreeMap::new();
    for (_, mut attrs) in xml_elements(part(parts, path)?, &["Relationship"])? {
        let mut required = |key| {
            attrs
                .remove(key)
                .ok_or_else(|| invalid("pptx", "incomplete relationship"))
        };
        let (id, kind, target) = (required("Id")?, required("Type")?, required("Target")?);
        if relations.insert(id, (kind, target)).is_some() {
            return Err(invalid("pptx", "duplicate relationship ID"));
        }
    }
    Ok(relations)
}

fn relationship_target<'a>(
    relations: &'a BTreeMap<String, (String, String)>,
    id: Option<&str>,
    kind: &str,
) -> ReportResult<&'a str> {
    let (actual, target) = id
        .and_then(|id| relations.get(id))
        .ok_or_else(|| invalid("pptx", "unresolved native relationship ID"))?;
    if actual
        .strip_prefix(pptx::REL)
        .and_then(|s| s.strip_prefix('/'))
        != Some(kind)
    {
        return Err(invalid("pptx", "unexpected native relationship type"));
    }
    Ok(target)
}

fn inspect_section_deck(
    parts: &BTreeMap<String, Vec<u8>>,
    presentation: &[u8],
    count: usize,
    chart_count: usize,
) -> ReportResult<bool> {
    let notes_masters = xml_elements(presentation, &["p:notesMasterId"])?;
    let [(_, master)] = notes_masters.as_slice() else {
        return Err(invalid("pptx", "missing section notes master"));
    };
    let list = xml_elements(presentation, &["p:sldId", "p:sldSz"])?;
    if count == 0 || list.len() != count + 1 {
        return Err(invalid("pptx", "invalid section slide list"));
    }
    let (size, attrs) = &list[count];
    if size != "p:sldSz"
        || attrs.get("cx").map(String::as_str) != Some("12192000")
        || attrs.get("cy").map(String::as_str) != Some("6858000")
    {
        return Err(invalid("pptx", "not 16:9 widescreen"));
    }
    let relations = relationships(parts, "ppt/_rels/presentation.xml.rels")?;
    let master = relationship_target(
        &relations,
        master.get("r:id").map(String::as_str),
        "notesMaster",
    )?;
    if master != "notesMasters/notesMaster1.xml" {
        return Err(invalid("pptx", "section notes master mismatch"));
    }
    let mut attached_charts = BTreeSet::new();
    let mut tables = false;
    for (index, (element, attrs)) in list[..count].iter().enumerate() {
        let number = index + 1;
        let target =
            relationship_target(&relations, attrs.get("r:id").map(String::as_str), "slide")?;
        if element != "p:sldId" || target != format!("slides/slide{number}.xml") {
            return Err(invalid("pptx", "section slide order mismatch"));
        }
        let rels = relationships(parts, &format!("ppt/slides/_rels/slide{number}.xml.rels"))?;
        let notes = relationship_target(&rels, Some("rIdNotes"), "notesSlide")?;
        if notes != format!("../notesSlides/notesSlide{number}.xml") {
            return Err(invalid("pptx", "section notes relationship mismatch"));
        }
        let notes_rels = relationships(
            parts,
            &format!("ppt/notesSlides/_rels/notesSlide{number}.xml.rels"),
        )?;
        if relationship_target(&notes_rels, Some("rIdSlide"), "slide")? != format!("../{target}")
            || relationship_target(&notes_rels, Some("rIdNotesMaster"), "notesMaster")?
                != format!("../{master}")
        {
            return Err(invalid(
                "pptx",
                "section notes reverse relationship mismatch",
            ));
        }
        let elements = xml_elements(
            part(parts, &format!("ppt/{target}"))?,
            &["p:sp", "a:t", "a:tbl", "c:chart", "a:hlinkClick"],
        )?;
        let has = |name| elements.iter().any(|(element, _)| element == name);
        if !has("p:sp") || !has("a:t") {
            return Err(invalid("pptx", "section slide has no editable heading"));
        }
        tables |= has("a:tbl");
        for (element, attrs) in &elements {
            match element.as_str() {
                "a:hlinkClick" => {
                    let target =
                        relationship_target(&rels, attrs.get("r:id").map(String::as_str), "slide")?;
                    let destination = target
                        .strip_prefix("slide")
                        .and_then(|s| s.strip_suffix(".xml"))
                        .and_then(|s| s.parse::<usize>().ok());
                    if !destination.is_some_and(|n| (1..=count).contains(&n))
                        || attrs.get("action").map(String::as_str)
                            != Some("ppaction://hlinksldjump")
                    {
                        return Err(invalid("pptx", "invalid section navigation"));
                    }
                }
                "c:chart" => {
                    let target =
                        relationship_target(&rels, attrs.get("r:id").map(String::as_str), "chart")?;
                    let chart = target
                        .strip_prefix("../charts/")
                        .and_then(|s| s.strip_suffix(".xml"))
                        .filter(|s| {
                            s.strip_prefix("chart")
                                .is_some_and(|n| n.parse::<usize>().is_ok())
                        })
                        .ok_or_else(|| invalid("pptx", "invalid attached chart target"))?;
                    if !attached_charts.insert(chart.to_owned()) {
                        return Err(invalid("pptx", "duplicate chart reference"));
                    }
                    inspect_chart_binding(parts, chart)?;
                }
                _ => {}
            }
        }
    }
    if attached_charts.len() != chart_count {
        return Err(invalid("pptx", "unattached chart part"));
    }
    Ok(tables)
}

fn inspect_chart_binding(parts: &BTreeMap<String, Vec<u8>>, name: &str) -> ReportResult<()> {
    let elements = xml_elements(
        part(parts, &format!("ppt/charts/{name}.xml"))?,
        &["c:lineChart", "c:dispBlanksAs", "c:externalData"],
    )?;
    let [(line, _), (gap, gap_attrs), (data, data_attrs)] = elements.as_slice() else {
        return Err(invalid(
            "pptx",
            "missing native line chart, gap mode or workbook link",
        ));
    };
    if line != "c:lineChart"
        || gap != "c:dispBlanksAs"
        || data != "c:externalData"
        || gap_attrs.get("val").map(String::as_str) != Some("gap")
    {
        return Err(invalid(
            "pptx",
            "missing native line chart, gap mode or workbook link",
        ));
    }
    let rels = relationships(parts, &format!("ppt/charts/_rels/{name}.xml.rels"))?;
    if relationship_target(&rels, data_attrs.get("r:id").map(String::as_str), "package")?
        != format!("../embeddings/{name}.xlsx")
    {
        return Err(invalid("pptx", "attached chart workbook mismatch"));
    }
    Ok(())
}

const THEME_STYLE_LISTS: [&[u8]; 4] = [
    b"fillStyleLst",
    b"lnStyleLst",
    b"effectStyleLst",
    b"bgFillStyleLst",
];

fn theme_style_child(list: usize, name: &[u8]) -> bool {
    match list {
        1 => name == b"ln",
        2 => name == b"effectStyle",
        _ => matches!(
            name,
            b"noFill" | b"solidFill" | b"gradFill" | b"blipFill" | b"pattFill" | b"grpFill"
        ),
    }
}

fn record_theme_style(
    stack: &[Vec<u8>],
    name: &[u8],
    drawing_namespace: bool,
    counts: &mut [Option<usize>; 4],
) -> ReportResult<()> {
    let in_matrix = stack.iter().take(3).map(Vec::as_slice).eq([
        b"theme".as_slice(),
        b"themeElements".as_slice(),
        b"fmtScheme".as_slice(),
    ]);
    if !in_matrix {
        return Ok(());
    }
    if stack.len() == 3 && drawing_namespace {
        if let Some(index) = THEME_STYLE_LISTS.iter().position(|list| *list == name) {
            if counts[index].replace(0).is_some() {
                return Err(invalid("pptx theme", "duplicate style matrix list"));
            }
        }
    } else if stack.len() == 4 {
        if let Some(index) = THEME_STYLE_LISTS.iter().position(|list| *list == stack[3]) {
            if !drawing_namespace || !theme_style_child(index, name) {
                return Err(invalid("pptx theme", "invalid style matrix child"));
            }
            *counts[index]
                .as_mut()
                .ok_or_else(|| invalid("pptx theme", "missing style matrix list"))? += 1;
        }
    }
    Ok(())
}

fn check_theme_style_matrix(data: &[u8]) -> ReportResult<()> {
    use quick_xml::{events::Event, name::ResolveResult};
    let mut reader = quick_xml::NsReader::from_reader(data);
    let mut stack = Vec::new();
    let mut counts = [None; 4];
    loop {
        let (namespace, event) = reader
            .read_resolved_event()
            .map_err(crate::OfficeError::from)?;
        let drawing = matches!(namespace, ResolveResult::Bound(ns) if ns.as_ref() == "http://schemas.openxmlformats.org/drawingml/2006/main");
        let opens = matches!(&event, Event::Start(_));
        match event {
            Event::Start(element) | Event::Empty(element) => {
                let name = element.local_name();
                record_theme_style(&stack, name.as_ref().as_bytes(), drawing, &mut counts)?;
                if opens {
                    if stack.len() >= 128 {
                        return Err(ReportError::LimitExceeded("theme XML nesting".into()));
                    }
                    stack.push(if drawing {
                        name.as_ref().as_bytes().to_vec()
                    } else {
                        Vec::new()
                    });
                }
            }
            Event::End(_) => {
                stack.pop();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if counts.iter().any(|count| count.unwrap_or(0) < 3) {
        return Err(invalid(
            "pptx theme",
            "style matrix lists require at least three entries",
        ));
    }
    Ok(())
}

fn parse_xml(data: &[u8]) -> ReportResult<()> {
    let mut reader = quick_xml::Reader::from_reader(data);
    let mut buf = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buf)
            .map_err(crate::OfficeError::from)?
        {
            quick_xml::events::Event::Eof => break,
            quick_xml::events::Event::DocType(_) => return Err(invalid("xml", "DTD is forbidden")),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}
fn check_rels(name: &str, data: &[u8], parts: &BTreeMap<String, Vec<u8>>) -> ReportResult<()> {
    let relpath = Path::new(name);
    let parent = relpath.parent().unwrap_or(Path::new(""));
    let source_dir = parent.parent().unwrap_or(Path::new(""));
    let mut reader = quick_xml::Reader::from_reader(data);
    let mut buf = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buf)
            .map_err(crate::OfficeError::from)?
        {
            quick_xml::events::Event::Empty(e) | quick_xml::events::Event::Start(e)
                if e.local_name().as_ref() == "Relationship" =>
            {
                let mut target = None;
                for a in e.attributes() {
                    let a = a.map_err(|_| invalid("relationships", "bad attribute"))?;
                    if a.key.as_ref() == "TargetMode" && a.value.as_ref() == "External" {
                        return Err(invalid("pptx", "external relationships forbidden"));
                    }
                    if a.key.as_ref() == "Target" {
                        target = Some(
                            a.normalized_value(quick_xml::XmlVersion::Explicit1_0)
                                .map_err(crate::OfficeError::from)?
                                .into_owned(),
                        );
                    }
                }
                let target = target.ok_or_else(|| invalid("relationships", "missing target"))?;
                let path = if let Some(abs) = target.strip_prefix('/') {
                    PathBuf::from(abs)
                } else {
                    source_dir.join(target)
                };
                let mut normal = PathBuf::new();
                for component in path.components() {
                    match component {
                        Component::ParentDir => {
                            if !normal.pop() {
                                return Err(invalid("relationships", "path escapes package"));
                            }
                        }
                        Component::CurDir => {}
                        Component::Normal(s) => normal.push(s),
                        _ => return Err(invalid("relationships", "invalid package target")),
                    }
                }
                let normal = normal
                    .to_str()
                    .ok_or_else(|| invalid("relationships", "non-UTF-8 path"))?;
                if !parts.contains_key(normal) {
                    return Err(invalid(
                        "relationships",
                        &format!("missing target {normal}"),
                    ));
                }
            }
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

const EXPANDED_PACKAGE_LIMIT: u64 = 128 * 1024 * 1024;
const PACKAGE_PART_LIMIT: usize = 10_000;

/// Nested workbooks consume the same resource budget as the outer presentation.
fn read_package(
    bytes: &[u8],
    expanded: &mut u64,
    part_count: &mut usize,
) -> ReportResult<BTreeMap<String, Vec<u8>>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    *part_count = part_count
        .checked_add(archive.len())
        .ok_or_else(|| ReportError::LimitExceeded("package parts".into()))?;
    if *part_count > PACKAGE_PART_LIMIT {
        return Err(ReportError::LimitExceeded("package parts".into()));
    }
    let mut parts = BTreeMap::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name()?.into_owned();
        if name.starts_with('/')
            || name.contains('\\')
            || name.split('/').any(|s| s == "..")
            || parts.contains_key(&name)
        {
            return Err(invalid("package", "unsafe or duplicate package path"));
        }
        let remaining = EXPANDED_PACKAGE_LIMIT.saturating_sub(*expanded);
        if entry.size() > remaining {
            return Err(ReportError::LimitExceeded("expanded package".into()));
        }
        let mut data = Vec::new();
        entry.by_ref().take(remaining + 1).read_to_end(&mut data)?;
        if data.len() as u64 > remaining {
            return Err(ReportError::LimitExceeded("expanded package".into()));
        }
        *expanded += data.len() as u64;
        parts.insert(name, data);
    }
    Ok(parts)
}

fn inspect_html(html: &str, document_sha256: &str, binding: Option<&str>) -> ReportResult<()> {
    use quick_xml::events::Event;
    const CSP: &str = "default-src 'none'; style-src 'unsafe-inline'; img-src data:; font-src data:; base-uri 'none'; form-action 'none'";
    const TAGS: &[&str] = &[
        "html",
        "head",
        "meta",
        "title",
        "style",
        "body",
        "main",
        "header",
        "p",
        "h1",
        "h2",
        "h3",
        "dl",
        "dt",
        "dd",
        "section",
        "nav",
        "a",
        "figure",
        "figcaption",
        "svg",
        "desc",
        "rect",
        "path",
        "text",
        "circle",
        "div",
        "table",
        "caption",
        "thead",
        "tr",
        "th",
        "tbody",
        "td",
        "footer",
        "ul",
        "li",
    ];
    const ATTRS: &[&str] = &[
        "class",
        "id",
        "href",
        "aria-label",
        "lang",
        "charset",
        "name",
        "content",
        "http-equiv",
        "viewbox",
        "role",
        "xmlns",
        "x",
        "y",
        "width",
        "height",
        "fill",
        "opacity",
        "d",
        "stroke",
        "stroke-dasharray",
        "stroke-width",
        "font-size",
        "cx",
        "cy",
        "r",
        "scope",
    ];
    if !html.starts_with("<!doctype html>")
        || !html.ends_with("</html>")
        || !html.contains(document_sha256)
    {
        return Err(invalid("html", "not the bound offline report document"));
    }
    let mut reader = quick_xml::Reader::from_str(html);
    // HTML's void meta elements do not have XML end tags. Tokenize attributes
    // structurally while enforcing the report profile, rather than searching
    // for case-sensitive active-tag substrings or a CSP mention in a comment.
    reader.config_mut().check_end_names = false;
    let mut in_head = false;
    let mut policy_seen = false;
    let mut marker = None;
    let mut first_content = None;
    let mut next_is_first_content = false;
    let mut ids = BTreeSet::new();
    let mut targets = Vec::new();
    loop {
        match reader.read_event().map_err(crate::OfficeError::from)? {
            Event::Start(e) | Event::Empty(e) => {
                let tag = e.name().as_ref().to_ascii_lowercase();
                if !TAGS.contains(&tag.as_str()) {
                    return Err(invalid("html", "unsupported or active HTML element"));
                }
                if next_is_first_content {
                    first_content = Some(tag.clone());
                    next_is_first_content = false;
                }
                if tag == "main" && first_content.is_none() {
                    next_is_first_content = true;
                }
                if tag == "head" {
                    in_head = true;
                }
                if matches!(tag.as_str(), "style" | "body") && !policy_seen {
                    return Err(invalid("html", "offline policy must precede content"));
                }
                let mut equivalent = None;
                let mut content = None;
                let mut meta_name = None;
                let mut names = BTreeSet::new();
                for attr in e.attributes() {
                    let attr = attr.map_err(|_| invalid("html", "bad HTML attribute"))?;
                    let name = attr.key.as_ref().to_ascii_lowercase();
                    if !names.insert(name.clone()) || !ATTRS.contains(&name.as_str()) {
                        return Err(invalid("html", "unsupported or active HTML attribute"));
                    }
                    let value = attr
                        .normalized_value(quick_xml::XmlVersion::Explicit1_0)
                        .map_err(crate::OfficeError::from)?
                        .into_owned();
                    if name == "id" {
                        if !matches!(tag.as_str(), "section" | "tr")
                            || value.trim().is_empty()
                            || !ids.insert(value.clone())
                        {
                            return Err(invalid("html", "invalid or duplicate internal anchor"));
                        }
                    }
                    if name == "href" {
                        let target = value.strip_prefix('#').filter(|target| !target.is_empty());
                        if tag != "a" || target.is_none() {
                            return Err(invalid(
                                "html",
                                "only internal report links are supported",
                            ));
                        }
                        targets.push(target.unwrap().to_owned());
                    }
                    if name == "http-equiv" {
                        equivalent = Some(value);
                    } else if name == "content" {
                        content = Some(value);
                    } else if name == "name" {
                        meta_name = Some(value);
                    }
                }
                if tag == "a" && !names.contains("href") {
                    return Err(invalid("html", "report link is missing its target"));
                }
                if meta_name.as_deref() == Some(html::BINDING_META) {
                    if tag != "meta" || !in_head || marker.is_some() {
                        return Err(invalid("html", "invalid embedded report binding"));
                    }
                    marker = Some(
                        content
                            .clone()
                            .ok_or_else(|| invalid("html", "missing embedded report binding"))?,
                    );
                }
                if let Some(equivalent) = equivalent {
                    if tag != "meta"
                        || !in_head
                        || policy_seen
                        || !equivalent.eq_ignore_ascii_case("Content-Security-Policy")
                        || content.as_deref() != Some(CSP)
                    {
                        return Err(invalid("html", "unsupported or missing offline policy"));
                    }
                    policy_seen = true;
                }
            }
            Event::End(e) => {
                if e.name().as_ref().eq_ignore_ascii_case("head") {
                    in_head = false;
                }
            }
            Event::DocType(value) if !value.as_ref().eq_ignore_ascii_case("html") => {
                return Err(invalid("html", "unsupported doctype"));
            }
            Event::PI(_) | Event::Decl(_) => {
                return Err(invalid("html", "unsupported HTML instruction"));
            }
            Event::Comment(_) | Event::CData(_) => {
                return Err(invalid(
                    "html",
                    "comments and CDATA are not part of the report profile",
                ));
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if targets.iter().any(|target| !ids.contains(target)) {
        return Err(invalid("html", "internal report link target is missing"));
    }
    if !policy_seen {
        return Err(invalid("html", "missing offline policy"));
    }
    verify_binding(marker.as_deref(), binding)?;
    if binding.is_some() && first_content.as_deref() != Some("section") {
        return Err(invalid("html", "first reading section is missing"));
    }
    Ok(())
}
