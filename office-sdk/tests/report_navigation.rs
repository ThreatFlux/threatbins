use office_sdk::report::*;
use std::io::{Cursor, Read};
use zip::ZipArchive;

fn document() -> ReportDocument {
    let rows=(0..40).map(|index|vec![format!("Gap {index}"),if index==0 {"Long retained detail. ".repeat(250)}else{format!("Recorded action for gap {index}; preserve the complete evidence and its next action.")}]).collect();
    ReportDocument {
        schema_version: 1,
        document_id: "navigation-contract".into(),
        revision: "1".into(),
        report_type: "practitioner".into(),
        title: "Retained findings".into(),
        subtitle: "Native internal navigation".into(),
        period: "Recorded period".into(),
        scope: "Caller-owned scope".into(),
        comparison: "Not applicable".into(),
        provenance: ReportProvenance {
            evidence_sha256: "a".repeat(64),
            sources: vec![],
        },
        definitions: vec![],
        limitations: vec![],
        sections: vec![
            ReportSection {
                id: "summary".into(),
                title: "Findings".into(),
                blocks: vec![ReportBlock::Links {
                    links: vec![
                        ReportLink {
                            label: "Review <gap> 39 & evidence".into(),
                            target_id: "gap:39".into(),
                        },
                        ReportLink {
                            label: "Follow the roadmap".into(),
                            target_id: "roadmap".into(),
                        },
                    ],
                }],
            },
            ReportSection {
                id: "gap-analysis".into(),
                title: "Complete gaps".into(),
                blocks: vec![ReportBlock::AnchoredTable {
                    table: ReportTable {
                        caption: "Recorded gaps".into(),
                        headers: vec!["Gap".into(), "Next action".into()],
                        rows,
                    },
                    row_ids: (0..40).map(|index| format!("gap:{index}")).collect(),
                }],
            },
            ReportSection {
                id: "roadmap".into(),
                title: "Roadmap".into(),
                blocks: vec![ReportBlock::Paragraph {
                    text: "Roadmap destination: verified retained actions.".into(),
                }],
            },
        ],
    }
}
fn part(bytes: &[u8], name: &str) -> String {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut text = String::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}

#[test]
fn internal_links_reach_exact_paginated_rows_in_all_native_formats() {
    let document = document();
    let digest = report_document_sha256(&document).unwrap();
    let html = render_report(&document, &ReportRenderOptions::new(ReportFormat::Html)).unwrap();
    let text = String::from_utf8(html.bytes).unwrap();
    assert!(text.contains("href=\"#gap:39\""));
    assert!(text.contains("<tr id=\"gap:39\">"));
    assert!(text.contains("<section id=\"roadmap\">"));
    assert!(text.contains("Review &lt;gap&gt; 39 &amp; evidence"));
    let pdf = render_report(&document, &ReportRenderOptions::new(ReportFormat::Pdf)).unwrap();
    let parsed = lopdf::Document::load_mem(&pdf.bytes).unwrap();
    let pages = parsed.get_pages();
    let mut targets = Vec::new();
    for id in pages.values() {
        let page = parsed.get_dictionary(*id).unwrap();
        let Ok(annotations) = page.get(b"Annots") else {
            continue;
        };
        for annotation in annotations.as_array().unwrap() {
            let value = match annotation {
                lopdf::Object::Reference(id) => parsed.get_dictionary(*id).unwrap(),
                lopdf::Object::Dictionary(value) => value,
                _ => panic!("a link annotation must be a dictionary or a dictionary reference"),
            };
            let action = value.get(b"A").unwrap().as_dict().unwrap();
            assert_eq!(action.get(b"S").unwrap().as_name().unwrap(), b"GoTo");
            let destination = action.get(b"D").unwrap().as_array().unwrap();
            let target = destination[0].as_reference().unwrap();
            let number = *pages.iter().find(|(_, id)| **id == target).unwrap().0;
            targets.push(parsed.extract_text(&[number]).unwrap());
        }
    }
    assert_eq!(targets.len(), 2);
    assert!(
        targets[0].contains("Gap 39"),
        "row target must survive preceding oversized rows and page splits"
    );
    assert!(targets[1].contains("Roadmap destination"));
    let pptx = render_report(&document, &ReportRenderOptions::new(ReportFormat::Pptx)).unwrap();
    let relationships = part(&pptx.bytes, "ppt/slides/_rels/slide2.xml.rels");
    let slide = part(&pptx.bytes, "ppt/slides/slide2.xml");
    assert!(slide.contains("<a:hlinkClick"));
    assert!(slide.contains("ppaction://hlinksldjump"));
    let mut reader = quick_xml::Reader::from_str(&relationships);
    let mut destinations = Vec::new();
    loop {
        match reader.read_event().unwrap() {
            quick_xml::events::Event::Empty(element)
                if element.local_name().as_ref() == "Relationship" =>
            {
                let attrs = element
                    .attributes()
                    .map(|attribute| {
                        let attribute = attribute.unwrap();
                        (
                            attribute.key.as_ref().to_owned(),
                            attribute.value.into_owned(),
                        )
                    })
                    .collect::<std::collections::BTreeMap<_, _>>();
                if attrs
                    .get("Id")
                    .is_some_and(|id| id.starts_with("rIdNavigation"))
                {
                    assert!(attrs["Type"].ends_with("/slide"));
                    destinations.push(part(
                        &pptx.bytes,
                        &format!("ppt/slides/{}", attrs["Target"]),
                    ));
                }
            }
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
    }
    assert_eq!(destinations.len(), 2);
    assert!(destinations[0].contains("Gap 39"));
    assert!(destinations[1].contains("Roadmap destination"));
    assert!(pptx.manifest.inspection.native_editable_tables);
    assert!(pptx.manifest.inspection.native_editable_text);
    assert_eq!(pdf.manifest.document_sha256, digest);
    assert_eq!(pptx.manifest.document_sha256, digest);
}

#[test]
fn navigation_rejects_missing_external_duplicate_and_empty_destinations() {
    let original = document();
    for target in [
        "missing",
        "https://example.invalid/remote",
        "javascript:alert(1)",
    ] {
        let mut value = original.clone();
        let ReportBlock::Links { links } = &mut value.sections[0].blocks[0] else {
            unreachable!()
        };
        links[0].target_id = target.into();
        assert!(validate_report_document(&value, &ReportLimits::default()).is_err());
    }
    let mut value = original.clone();
    let ReportBlock::AnchoredTable { row_ids, .. } = &mut value.sections[1].blocks[0] else {
        unreachable!()
    };
    row_ids[0] = "roadmap".into();
    assert!(validate_report_document(&value, &ReportLimits::default()).is_err());
    let mut value = original.clone();
    value.sections[2].blocks.clear();
    assert!(validate_report_document(&value, &ReportLimits::default()).is_err());
    let mut value = original;
    let ReportBlock::AnchoredTable { row_ids, .. } = &mut value.sections[1].blocks[0] else {
        unreachable!()
    };
    row_ids.pop();
    assert!(validate_report_document(&value, &ReportLimits::default()).is_err());
}

#[test]
fn inspection_rejects_tampered_external_and_dangling_html_links() {
    use sha2::{Digest, Sha256};
    let rendered =
        render_report(&document(), &ReportRenderOptions::new(ReportFormat::Html)).unwrap();
    let html = String::from_utf8(rendered.bytes.clone()).unwrap();
    for replacement in [
        "href=\"https://example.invalid/\"",
        "href=\"javascript:alert(1)\"",
        "href=\"#missing\"",
    ] {
        let bytes = html
            .replacen("href=\"#gap:39\"", replacement, 1)
            .into_bytes();
        let mut manifest = rendered.manifest.clone();
        manifest.byte_size = bytes.len();
        manifest.output_sha256 = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert!(inspect_report(&bytes, &manifest).is_err());
    }
}
