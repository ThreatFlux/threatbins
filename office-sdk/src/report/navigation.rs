//! Internal document navigation. Targets are caller-owned facts, never fetched URLs.
use super::*;
use std::collections::HashSet;

pub(super) fn validate(document: &ReportDocument) -> ReportResult<()> {
    let mut reserved = HashSet::new();
    let mut targets = HashSet::new();
    for section in &document.sections {
        if !reserved.insert(section.id.as_str()) {
            return Err(invalid("navigation", "duplicate anchor"));
        }
        if !section.blocks.is_empty() {
            targets.insert(section.id.as_str());
        }
    }
    for block in document.sections.iter().flat_map(|section| &section.blocks) {
        if let ReportBlock::AnchoredTable { table, row_ids } = block {
            if row_ids.len() != table.rows.len() {
                return Err(invalid(
                    "row_ids",
                    "one anchor is required for each table row",
                ));
            }
            for id in row_ids {
                if id.trim().is_empty() || id.len() > 256 || !reserved.insert(id.as_str()) {
                    return Err(invalid(
                        "row_ids",
                        "anchors must be short, nonempty and unique throughout the document",
                    ));
                }
                targets.insert(id.as_str());
            }
        }
    }
    for block in document.sections.iter().flat_map(|section| &section.blocks) {
        if let ReportBlock::Links { links } = block {
            if links.is_empty() {
                return Err(invalid("links", "at least one internal link is required"));
            }
            for link in links {
                if link.label.trim().is_empty() || !targets.contains(link.target_id.as_str()) {
                    return Err(invalid(
                        "links",
                        "a nonempty label and an existing internal content target are required",
                    ));
                }
            }
        }
    }
    Ok(())
}
