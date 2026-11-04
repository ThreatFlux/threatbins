use super::*;
use std::{
    collections::BTreeMap,
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
    if manifest.schema_version != REPORT_SCHEMA_VERSION
        || manifest.template != REPORT_TEMPLATE
        || manifest.mime_type != manifest.format.mime_type()
    {
        return Err(invalid(
            "manifest",
            "unsupported schema/template or MIME mismatch",
        ));
    }
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
            result
                .structural_checks
                .push("PDF parsed and page count matches manifest".into());
        }
        ReportFormat::Html => {
            let h = std::str::from_utf8(bytes).map_err(|_| invalid("html", "not UTF-8"))?;
            inspect_html(h, &manifest.document_sha256)?;
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
            result.native_editable_text = parts.iter().any(|(n, v)| {
                n.starts_with("ppt/slides/slide") && String::from_utf8_lossy(v).contains("<a:t")
            });
            result.native_editable_tables = parts.iter().any(|(n, v)| {
                n.starts_with("ppt/slides/slide") && String::from_utf8_lossy(v).contains("<a:tbl>")
            });
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
                "chart parts reference embedded editable workbook data".into(),
            ]);
        }
    }
    result
        .structural_checks
        .push("byte size and SHA-256 match manifest".into());
    result.visual_review_required = true;
    Ok(result)
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

fn inspect_html(html: &str, document_sha256: &str) -> ReportResult<()> {
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
    loop {
        match reader.read_event().map_err(crate::OfficeError::from)? {
            Event::Start(e) | Event::Empty(e) => {
                let tag = e.name().as_ref().to_ascii_lowercase();
                if !TAGS.contains(&tag.as_str()) {
                    return Err(invalid("html", "unsupported or active HTML element"));
                }
                if tag == "head" {
                    in_head = true;
                }
                if matches!(tag.as_str(), "style" | "body") && !policy_seen {
                    return Err(invalid("html", "offline policy must precede content"));
                }
                let mut equivalent = None;
                let mut content = None;
                for attr in e.attributes() {
                    let attr = attr.map_err(|_| invalid("html", "bad HTML attribute"))?;
                    let name = attr.key.as_ref().to_ascii_lowercase();
                    if !ATTRS.contains(&name.as_str()) {
                        return Err(invalid("html", "unsupported or active HTML attribute"));
                    }
                    let value = attr
                        .normalized_value(quick_xml::XmlVersion::Explicit1_0)
                        .map_err(crate::OfficeError::from)?
                        .into_owned();
                    if name == "http-equiv" {
                        equivalent = Some(value);
                    } else if name == "content" {
                        content = Some(value);
                    }
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
            Event::Eof => break,
            _ => {}
        }
    }
    if !policy_seen {
        return Err(invalid("html", "missing offline policy"));
    }
    Ok(())
}
