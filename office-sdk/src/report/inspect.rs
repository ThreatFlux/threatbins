use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
    path::{Component, Path, PathBuf},
};

/// Verify returned bytes against an SDK manifest and inspect structural parts.
/// This does not replace visual review or establish application approval.
pub fn inspect_report(bytes: &[u8], manifest: &ReportManifest) -> ReportResult<ReportInspection> {
    inspect_with_html_font(bytes, manifest, None)
}

// Rendering can check the embedded font in the same pass that reads the HTML.
pub(super) fn inspect_with_html_font(
    bytes: &[u8],
    manifest: &ReportManifest,
    html_font: Option<&layout::FontMetrics>,
) -> ReportResult<ReportInspection> {
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
            inspect_html(h, &manifest.document_sha256, binding.as_deref(), html_font)?;
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

// Keep the exact ancestor chain for the section-first structures whose
// meaning depends on where the element occurs.
fn walk_report_xml<'a>(
    data: &'a [u8],
    mut visit: impl FnMut(&[String], quick_xml::events::Event<'a>) -> ReportResult<()>,
) -> ReportResult<()> {
    use quick_xml::{events::Event, name::ResolveResult};
    let mut reader = quick_xml::NsReader::from_reader(data);
    let mut parents = Vec::new();
    let mut root_seen = false;
    loop {
        let event = reader.read_event().map_err(crate::OfficeError::from)?;
        match &event {
            Event::Start(element) | Event::Empty(element) => {
                if parents.is_empty() && std::mem::replace(&mut root_seen, true) {
                    return Err(invalid("pptx", "multiple report XML roots"));
                }
                let name = element.name();
                let namespace = match name.as_ref().split_once(':').map(|(prefix, _)| prefix) {
                    Some("p") => "http://schemas.openxmlformats.org/presentationml/2006/main",
                    Some("a") => "http://schemas.openxmlformats.org/drawingml/2006/main",
                    Some("c") => "http://schemas.openxmlformats.org/drawingml/2006/chart",
                    _ => "",
                };
                let (actual, _) = reader.resolver().resolve_element(name);
                if !namespace.is_empty()
                    && !matches!(actual, ResolveResult::Bound(uri) if uri.as_ref() == namespace)
                {
                    return Err(invalid("pptx", "unexpected OOXML element namespace"));
                }
                if matches!(event, Event::Start(_)) {
                    if parents.len() >= 128 {
                        return Err(ReportError::LimitExceeded("report XML nesting".into()));
                    }
                    let name = name.as_ref().to_owned();
                    visit(&parents, event)?;
                    parents.push(name);
                    continue;
                }
            }
            Event::End(_) => {
                parents.pop();
            }
            Event::Eof => {
                return if root_seen && parents.is_empty() {
                    Ok(())
                } else {
                    Err(invalid("pptx", "incomplete report XML"))
                };
            }
            _ => {}
        }
        visit(&parents, event)?;
    }
}

fn node_attribute(
    element: &quick_xml::events::BytesStart<'_>,
    key: &str,
) -> ReportResult<Option<String>> {
    element
        .try_get_attribute(key)
        .map_err(|_| invalid("pptx", "invalid attribute"))?
        .map(|attribute| {
            attribute
                .normalized_value(quick_xml::XmlVersion::Explicit1_0)
                .map(|value| value.into_owned())
                .map_err(|error| ReportError::from(crate::OfficeError::from(error)))
        })
        .transpose()
}

#[derive(Default)]
struct ShapeText {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    text_box: bool,
    notes_body: bool,
    paragraphs: Vec<String>,
    preceding_table_rows: Option<usize>,
}

fn report_shape_text(data: &[u8], root: &str) -> ReportResult<Vec<ShapeText>> {
    use quick_xml::events::Event;
    let base = [root, "p:cSld", "p:spTree"];
    let mut shapes = Vec::new();
    let mut current = None;
    let mut table_rows = None;
    let mut table_graphic = false;
    walk_report_xml(data, |parents, event| {
        if parents
            .get(..base.len())
            .is_none_or(|prefix| prefix != base)
        {
            return Ok(());
        }
        let path = &parents[base.len()..];
        if path.is_empty() {
            // A preview caption is the next shape after its native table.
            match event {
                Event::Start(e) if e.name().as_ref() == "p:sp" => {
                    current = Some(ShapeText {
                        preceding_table_rows: table_rows.take(),
                        ..Default::default()
                    });
                }
                Event::End(e) if e.name().as_ref() == "p:sp" => {
                    shapes.extend(current.take());
                }
                Event::Start(_) | Event::Empty(_) => {
                    table_rows = None;
                    table_graphic = false;
                }
                _ => {}
            }
            return Ok(());
        }
        let Some(shape) = &mut current else {
            if root == "p:sld"
                && let Event::Start(element) | Event::Empty(element) = event
            {
                match element.name().as_ref() {
                    "a:graphicData" if path == ["p:graphicFrame", "a:graphic"] => {
                        table_graphic = node_attribute(&element, "uri")?.as_deref()
                            == Some(pptx::TABLE_GRAPHIC_URI);
                    }
                    "a:tbl"
                        if table_graphic
                            && path == ["p:graphicFrame", "a:graphic", "a:graphicData"] =>
                    {
                        if table_rows.replace(0).is_some() {
                            return Err(invalid("pptx", "multiple tables in one graphic frame"));
                        }
                    }
                    "a:tr"
                        if table_graphic
                            && path
                                == ["p:graphicFrame", "a:graphic", "a:graphicData", "a:tbl"]
                            && let Some(rows) = &mut table_rows =>
                    {
                        *rows += 1;
                    }
                    _ => {}
                }
            }
            return Ok(());
        };
        let opens = matches!(&event, Event::Start(_));
        match event {
            Event::Start(element) | Event::Empty(element) => match element.name().as_ref() {
                "p:cNvPr" if path == ["p:sp", "p:nvSpPr"] => {
                    shape.id = node_attribute(&element, "id")?;
                    shape.name = node_attribute(&element, "name")?;
                    shape.description = node_attribute(&element, "descr")?;
                }
                "p:cNvSpPr" if path == ["p:sp", "p:nvSpPr"] => {
                    shape.text_box = matches!(
                        node_attribute(&element, "txBox")?.as_deref(),
                        Some("1" | "true")
                    );
                }
                "p:ph" if path == ["p:sp", "p:nvSpPr", "p:nvPr"] => {
                    shape.notes_body = node_attribute(&element, "type")?.as_deref() == Some("body")
                        && node_attribute(&element, "idx")?.as_deref() == Some("1");
                }
                "a:p" if path == ["p:sp", "p:txBody"] && (opens || root == "p:notes") => {
                    shape.paragraphs.push(String::new());
                }
                _ => {}
            },
            text if path == ["p:sp", "p:txBody", "a:p", "a:r", "a:t"] => {
                let Some(paragraph) = shape.paragraphs.last_mut() else {
                    return Ok(());
                };
                match text {
                    Event::Text(text) => paragraph.push_str(&crate::utils::decode_xml_text(&text)?),
                    Event::GeneralRef(reference) => {
                        paragraph.push_str(&crate::utils::decode_xml_reference(&reference)?)
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    })?;
    Ok(shapes)
}

fn meaningful_heading(text: &str) -> bool {
    text.chars().any(|c| {
        !c.is_whitespace()
            && !c.is_control()
            && !matches!(c, '\u{200b}'..='\u{200f}' | '\u{2060}' | '\u{feff}')
    })
}

fn inspect_slide_text(slide: &[u8], notes: &[u8]) -> ReportResult<()> {
    let shapes = report_shape_text(slide, "p:sld")?;
    let mut headings = shapes
        .iter()
        .filter(|shape| shape.id.as_deref() == Some("2"));
    let heading = headings.next();
    if headings.next().is_some()
        || !heading.is_some_and(|shape| {
            shape.text_box && shape.paragraphs.iter().any(|text| meaningful_heading(text))
        })
    {
        return Err(invalid("pptx", "section slide has no editable heading"));
    }

    let previews = shapes
        .iter()
        .filter_map(|shape| {
            let marker = shape
                .description
                .as_deref()
                .and_then(|description| description.strip_prefix(section::TABLE_PREVIEW_PREFIX));
            // The shape name still identifies a preview when its binding is damaged.
            (shape.name.as_deref() == Some(pptx::TABLE_PREVIEW_SHAPE_NAME) || marker.is_some())
                .then_some((shape, marker))
        })
        .collect::<Vec<_>>();
    if previews.is_empty() {
        return Ok(());
    }
    let notes = report_shape_text(notes, "p:notes")?;
    let mut bodies = notes.iter().filter(|shape| shape.notes_body);
    let Some(body) = bodies.next().filter(|_| bodies.next().is_none()) else {
        return Err(invalid(
            "pptx",
            "table preview requires a unique notes body",
        ));
    };
    let mut previous_end = 0;
    for (shape, marker) in previews {
        let text = shape.paragraphs.join(" ");
        previous_end = inspect_table_preview(
            &body.paragraphs,
            &text,
            marker,
            previous_end,
            shape.preceding_table_rows,
        )?;
    }
    Ok(())
}

fn inspect_table_preview(
    paragraphs: &[String],
    text: &str,
    marker: Option<&str>,
    previous_end: usize,
    table_rows: Option<usize>,
) -> ReportResult<usize> {
    let error = || {
        invalid(
            "pptx",
            "table preview does not match its complete table in speaker notes",
        )
    };
    let mut fields = marker.ok_or_else(error)?.split(':');
    let (visible, total, columns, start, length) = (
        fields
            .next()
            .and_then(|v| v.parse::<usize>().ok())
            .ok_or_else(error)?,
        fields
            .next()
            .and_then(|v| v.parse::<usize>().ok())
            .ok_or_else(error)?,
        fields
            .next()
            .and_then(|v| v.parse::<usize>().ok())
            .ok_or_else(error)?,
        fields
            .next()
            .and_then(|v| v.parse::<usize>().ok())
            .ok_or_else(error)?,
        fields
            .next()
            .and_then(|v| v.parse::<usize>().ok())
            .ok_or_else(error)?,
    );
    let digest = fields.next().ok_or_else(error)?;
    if table_rows.and_then(|rows| rows.checked_sub(1)) != Some(visible) {
        return Err(invalid(
            "pptx",
            "table preview does not match the preceding native table row count",
        ));
    }
    let end = start.checked_add(length).ok_or_else(error)?;
    let record = paragraphs.get(start..end).ok_or_else(error)?;
    let row_count = format!("Rows: {total}");
    let mut rows = record.iter();
    if fields.next().is_some()
        || visible >= total
        || total > ReportLimits::default().max_table_rows
        || !(1..=8).contains(&columns)
        || start < previous_end
        || length < 2 + total * (1 + columns)
        || text
            != format!(
                "{visible} of {total}{}",
                section::TABLE_PREVIEW_CAPTION_SUFFIX
            )
        || !record
            .first()
            .is_some_and(|line| line.starts_with("Complete table: "))
        || !rows.any(|line| line == &row_count)
        || sha256(record.join("\n").as_bytes()) != digest
    {
        return Err(error());
    }
    for number in 1..=total {
        let expected = format!("Row {number}");
        if !rows.any(|line| {
            line.strip_prefix(&expected)
                .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with(" ("))
        }) {
            return Err(error());
        }
        for _ in 0..columns {
            if !rows.any(|line| line.contains(": ")) {
                return Err(error());
            }
        }
    }
    Ok(end)
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
        let slide = part(parts, &format!("ppt/{target}"))?;
        let notes_body = part(parts, &format!("ppt/notesSlides/notesSlide{number}.xml"))?;
        inspect_slide_text(slide, notes_body)?;
        let elements = xml_elements(slide, &["a:tbl", "c:chart", "a:hlinkClick"])?;
        tables |= elements.iter().any(|(element, _)| element == "a:tbl");
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
    use quick_xml::events::Event;
    let chart = part(parts, &format!("ppt/charts/{name}.xml"))?;
    let mut counts = [0; 6];
    walk_report_xml(chart, |parents, event| {
        let opens = matches!(&event, Event::Start(_));
        let (Event::Start(element) | Event::Empty(element)) = event else {
            return Ok(());
        };
        let (index, required): (usize, &[&str]) = match element.name().as_ref() {
            "c:chartSpace" => (0, &[]),
            "c:chart" => (1, &["c:chartSpace"]),
            "c:plotArea" => (2, &["c:chartSpace", "c:chart"]),
            "c:lineChart" => (3, &["c:chartSpace", "c:chart", "c:plotArea"]),
            "c:dispBlanksAs" => (4, &["c:chartSpace", "c:chart"]),
            "c:externalData" => (5, &["c:chartSpace"]),
            _ => return Ok(()),
        };
        if parents != required || (index < 4 && !opens) || (index == 4 && counts[2] != 1) {
            return Err(invalid("pptx", "unexpected native chart element position"));
        }
        counts[index] += 1;
        Ok(())
    })?;
    if counts != [1; 6] {
        return Err(invalid(
            "pptx",
            "missing or duplicate native chart structure",
        ));
    }
    // Also check the relationship attribute namespace and the explicit gap value.
    let elements = xml_elements(chart, &["c:dispBlanksAs", "c:externalData"])?;
    let [(gap, gap_attrs), (data, data_attrs)] = elements.as_slice() else {
        return Err(invalid(
            "pptx",
            "missing native chart gap mode or workbook link",
        ));
    };
    if gap != "c:dispBlanksAs"
        || data != "c:externalData"
        || gap_attrs.get("val").map(String::as_str) != Some("gap")
    {
        return Err(invalid(
            "pptx",
            "missing native chart gap mode or workbook link",
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

#[derive(Default)]
struct HtmlReadingOrder {
    stack: Vec<String>,
    main_seen: bool,
    opening_seen: bool,
    in_opening: bool,
    opening_heading_seen: bool,
    heading_has_text: bool,
}

impl HtmlReadingOrder {
    fn start(&mut self, tag: &str, opens: bool) -> ReportResult<()> {
        let parent = self.stack.last().map(String::as_str);
        if tag == "main" {
            if self.main_seen || !opens || parent != Some("body") {
                return Err(invalid("html", "invalid report main"));
            }
            self.main_seen = true;
        }
        if tag == "section" && !self.stack.iter().any(|name| name == "main") {
            return Err(invalid("html", "report section is outside main"));
        }
        if parent == Some("main") && !self.opening_seen {
            if tag != "section" || !opens {
                return Err(invalid("html", "first reading section is missing"));
            }
            self.opening_seen = true;
            self.in_opening = true;
        } else if self.in_opening
            && !self.opening_heading_seen
            && (parent != Some("section") || tag != "h1" || !opens)
        {
            return Err(invalid(
                "html",
                "first reading section must start with an h1",
            ));
        }
        if parent == Some("figcaption")
            && matches!(tag, "h1" | "h2" | "h3")
            && tag != if self.in_opening { "h2" } else { "h3" }
        {
            return Err(invalid("html", "incorrect chart heading level"));
        }
        if opens && tag != "meta" {
            if self.stack.len() >= 128 {
                return Err(ReportError::LimitExceeded("HTML nesting".into()));
            }
            self.stack.push(tag.into());
        }
        Ok(())
    }

    fn end(&mut self, tag: &str) -> ReportResult<()> {
        if self.stack.pop().as_deref() != Some(tag) {
            return Err(invalid("html", "malformed section-first reading order"));
        }
        if tag == "main" && !self.opening_seen {
            return Err(invalid("html", "first reading section is missing"));
        }
        if self.in_opening && !self.opening_heading_seen {
            if tag != "h1" || !self.heading_has_text {
                return Err(invalid(
                    "html",
                    "first reading section requires a meaningful h1",
                ));
            }
            self.opening_heading_seen = true;
        }
        if tag == "section" && self.stack.last().is_some_and(|parent| parent == "main") {
            self.in_opening = false;
        }
        Ok(())
    }

    fn checks_opening_text(&self) -> bool {
        !self.opening_heading_seen
            && !matches!(self.stack.as_slice(),
                [html, head, tag] if html == "html" && head == "head" && matches!(tag.as_str(), "title" | "style"))
    }

    fn text_font<'a>(
        &self,
        font: Option<&'a layout::FontMetrics>,
    ) -> Option<&'a layout::FontMetrics> {
        font.filter(|_| self.stack.iter().any(|tag| tag == "body"))
    }

    fn text(&mut self, text: &str, font: Option<&layout::FontMetrics>) -> ReportResult<()> {
        if self.checks_opening_text() {
            if self.in_opening && self.stack.last().is_some_and(|tag| tag == "h1") {
                self.heading_has_text |= meaningful_heading(text);
            } else if !text.trim().is_empty() {
                let position = if self.opening_seen {
                    "text precedes the opening h1"
                } else {
                    "text precedes the first reading section"
                };
                return Err(invalid("html", position));
            }
        }
        if let Some(font) = self.text_font(font) {
            font.width(text, 1.0)?;
        }
        Ok(())
    }

    fn finish(self) -> ReportResult<()> {
        if !self.main_seen || !self.opening_heading_seen || !self.stack.is_empty() {
            return Err(invalid("html", "first reading section is missing"));
        }
        Ok(())
    }
}

fn inspect_html(
    html: &str,
    document_sha256: &str,
    binding: Option<&str>,
    font: Option<&layout::FontMetrics>,
) -> ReportResult<()> {
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
    let mut reading_order = binding.map(|_| HtmlReadingOrder::default());
    let mut ids = BTreeSet::new();
    let mut targets = Vec::new();
    loop {
        let event = reader.read_event().map_err(crate::OfficeError::from)?;
        let opens = matches!(&event, Event::Start(_));
        match event {
            Event::Start(e) | Event::Empty(e) => {
                let tag = e.name().as_ref().to_ascii_lowercase();
                if !TAGS.contains(&tag.as_str()) {
                    return Err(invalid("html", "unsupported or active HTML element"));
                }
                if let Some(reading_order) = &mut reading_order {
                    reading_order.start(&tag, opens)?;
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
                let tag = e.name().as_ref().to_ascii_lowercase();
                if tag == "head" {
                    in_head = false;
                }
                if let Some(reading_order) = &mut reading_order {
                    reading_order.end(&tag)?;
                }
            }
            Event::Text(text) => {
                if let Some(order) = &mut reading_order {
                    order.text(text.as_ref(), font)?;
                }
            }
            Event::GeneralRef(reference) => {
                if let Some(order) = &mut reading_order
                    && (order.checks_opening_text() || order.text_font(font).is_some())
                {
                    order.text(&crate::utils::decode_xml_reference(&reference)?, font)?;
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
    if let Some(reading_order) = reading_order {
        reading_order.finish()?;
    }
    Ok(())
}
