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
            if !h.starts_with("<!doctype html>")
                || !h.ends_with("</html>")
                || !h.contains("Content-Security-Policy")
                || h.contains("<script")
                || h.contains("<iframe")
                || h.contains("<object")
                || h.contains("<link")
            {
                return Err(invalid("html", "not the offline safe report document"));
            }
            if !h.contains(&manifest.document_sha256) {
                return Err(invalid("html", "missing document binding"));
            }
            result
                .structural_checks
                .push("UTF-8 offline HTML with document binding and no active content".into());
        }
        ReportFormat::Pptx => {
            let mut z = zip::ZipArchive::new(Cursor::new(bytes))?;
            if z.len() > 10_000 {
                return Err(ReportError::LimitExceeded("PPTX parts".into()));
            }
            let mut parts = BTreeMap::new();
            let mut expanded = 0u64;
            for i in 0..z.len() {
                let mut entry = z.by_index(i)?;
                expanded = expanded
                    .checked_add(entry.size())
                    .ok_or_else(|| ReportError::LimitExceeded("expanded PPTX".into()))?;
                if expanded > 128 * 1024 * 1024 {
                    return Err(ReportError::LimitExceeded("expanded PPTX".into()));
                }
                let name = entry.name()?.into_owned();
                if name.starts_with('/')
                    || name.split('/').any(|s| s == "..")
                    || parts.contains_key(&name)
                {
                    return Err(invalid("pptx", "unsafe or duplicate package path"));
                }
                let mut data = Vec::new();
                entry.read_to_end(&mut data)?;
                parts.insert(name, data);
            }
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
                    let mut workbook = zip::ZipArchive::new(Cursor::new(data))?;
                    let mut sheet = String::new();
                    workbook
                        .by_name("xl/worksheets/sheet1.xml")?
                        .read_to_string(&mut sheet)?;
                    parse_xml(sheet.as_bytes())?;
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
