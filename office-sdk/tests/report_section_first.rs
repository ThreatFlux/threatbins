use office_sdk::report::*;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read, Write};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const FIXTURE: &str = include_str!("fixtures/section-first-report.json");
const DENSE: &str = include_str!("fixtures/dense-section-first-report.json");
const APPENDIX: &str = include_str!("fixtures/navigation-report.json");
const FORMATS: [ReportFormat; 3] = [ReportFormat::Pdf, ReportFormat::Html, ReportFormat::Pptx];

fn options(format: ReportFormat) -> ReportRenderOptions {
    let mut options = ReportRenderOptions::new(format);
    options.template = SECTION_FIRST_TEMPLATE.into();
    options
}

fn fixture() -> ReportDocument {
    serde_json::from_str(FIXTURE).unwrap()
}

fn part(bytes: &[u8], path: &str) -> Vec<u8> {
    let mut zip = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut output = Vec::new();
    zip.by_name(path).unwrap().read_to_end(&mut output).unwrap();
    output
}

fn xml(bytes: &[u8], path: &str) -> String {
    String::from_utf8(part(bytes, path)).unwrap()
}

fn assert_in_order(haystack: &str, first: &str, second: &str) {
    let first = haystack.find(first).expect("missing first text");
    let second = haystack.find(second).expect("missing second text");
    assert!(first < second, "incorrect reading order");
}

#[test]
fn section_first_is_opt_in_and_preserves_schema_one() {
    let document = fixture();
    let digest = report_document_sha256(&document).unwrap();
    assert_eq!(report_capabilities().schema_version, 1);
    assert_eq!(
        report_capabilities().templates,
        [REPORT_TEMPLATE, SECTION_FIRST_TEMPLATE]
    );
    for format in FORMATS {
        let default = ReportRenderOptions::new(format);
        assert_eq!(default.template, REPORT_TEMPLATE);
        let legacy = render_report(&document, &default).unwrap();
        let mut selected = options(format);
        selected.expected_document_sha256 = Some(digest.clone());
        let rendered = render_report(&document, &selected).unwrap();
        let manifest = &rendered.manifest;
        assert_eq!(legacy.manifest.document_sha256, digest);
        assert_eq!(manifest.document_sha256, digest);
        assert_eq!(
            manifest.evidence_sha256,
            document.provenance.evidence_sha256
        );
        assert_eq!(manifest.revision, document.revision);
        assert_eq!(manifest.document_id, document.document_id);
        assert_eq!(manifest.sdk_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.template, SECTION_FIRST_TEMPLATE);
        assert_eq!(manifest.format, format);
        assert_eq!(manifest.mime_type, format.mime_type());
        assert_eq!(manifest.byte_size, rendered.bytes.len());
        assert!(manifest.inspection.visual_review_required);
        let inspection = inspect_report(&rendered.bytes, manifest).unwrap();
        assert_eq!(
            inspection.structural_checks,
            manifest.inspection.structural_checks
        );
        let editable = format == ReportFormat::Pptx;
        assert_eq!(inspection.native_editable_text, editable);
        assert_eq!(inspection.native_editable_tables, editable);
        assert_eq!(inspection.native_editable_charts, editable);

        match format {
            ReportFormat::Pdf => {
                let pdf = lopdf::Document::load_mem(&rendered.bytes).unwrap();
                let legacy = lopdf::Document::load_mem(&legacy.bytes).unwrap();
                let first = pdf.extract_text(&[1]).unwrap();
                let all = pdf
                    .extract_text(&pdf.get_pages().into_keys().collect::<Vec<_>>())
                    .unwrap();
                assert_in_order(
                    &first,
                    "Bottom line",
                    "Measured coverage reached 78 percent",
                );
                assert_in_order(
                    &first,
                    "Measured coverage reached 78 percent",
                    "Previous period",
                );
                assert_in_order(
                    &all,
                    "Prioritize the outstanding sampling work",
                    "Report context",
                );
                assert!(first.contains("April to June 2026"));
                assert!(
                    legacy
                        .extract_text(&[1])
                        .unwrap()
                        .contains("Report overview")
                );
                for item in ["Not measured", "Definitions", "observation-log @ rev-4"] {
                    assert!(all.contains(item), "PDF lost {item}");
                }
            }
            ReportFormat::Html => {
                let html = std::str::from_utf8(&rendered.bytes).unwrap();
                let legacy = std::str::from_utf8(&legacy.bytes).unwrap();
                assert!(html.contains("<main><section id=\"bottom-line\"><h1>Bottom line</h1>"));
                assert!(legacy.contains("<main><header>"));
                assert_in_order(
                    html,
                    "Prioritize the outstanding sampling work",
                    "<dt>Period</dt>",
                );
                assert!(html.contains("href=\"#receiving-action\""));
                assert!(html.contains("<tr id=\"receiving-action\">"));
                assert!(html.contains(
                    "Opening observation in the northern distribution and staging corridor"
                ));
                assert!(html.contains("<td>Not measured</td>"));
                assert!(html.contains("observation-log @ rev-4"));
                assert_eq!(manifest.pages_or_slides, None);
            }
            ReportFormat::Pptx => {
                assert_eq!(manifest.pages_or_slides, Some(document.sections.len()));
                assert!(legacy.manifest.pages_or_slides.unwrap() > document.sections.len());
                let first = xml(&rendered.bytes, "ppt/slides/slide1.xml");
                assert_in_order(
                    &first,
                    "Bottom line",
                    "Measured coverage reached 78 percent",
                );
                assert_in_order(
                    &first,
                    "Prioritize the outstanding sampling work",
                    "Previous period",
                );
                assert!(!first.contains("Report overview"));
                assert!(xml(&legacy.bytes, "ppt/slides/slide1.xml").contains("Report overview"));
            }
        }
    }
    let mut unknown = options(ReportFormat::Pptx);
    unknown.template = "future_template".into();
    assert!(matches!(
        render_report(&document, &unknown),
        Err(ReportError::UnsupportedTemplate(_))
    ));
}

#[test]
fn dense_sections_keep_native_cells_workbooks_gaps_and_links() {
    let document = fixture();
    let report = render_report(&document, &options(ReportFormat::Pptx)).unwrap();
    let slide = xml(&report.bytes, "ppt/slides/slide2.xml");
    let slide1 = xml(&report.bytes, "ppt/slides/slide1.xml");
    let rels = xml(&report.bytes, "ppt/slides/_rels/slide1.xml.rels");
    assert!(slide1.contains("ppaction://hlinksldjump"));
    assert!(rels.contains("Target=\"slide4.xml\""));
    assert!(rels.contains("Target=\"slide2.xml\""));
    assert!(slide.contains("<a:tbl>"));
    assert_eq!(slide.matches("<a:tr ").count(), 13);
    assert!(slide.contains("Packing review"));
    assert!(slide.contains("Not measured"));
    assert!(slide.contains("<c:chart "));
    let presentation = xml(&report.bytes, "ppt/presentation.xml");
    assert!(presentation.contains("cx=\"12192000\" cy=\"6858000\""));
    let paired = xml(&report.bytes, "ppt/slides/slide3.xml");
    assert_eq!(paired.matches("<c:chart ").count(), 2);
    let paired_rels = xml(&report.bytes, "ppt/slides/_rels/slide3.xml.rels");
    assert!(paired_rels.contains("rIdChart2"));
    assert!(paired_rels.contains("rIdChart3"));
    let chart = xml(&report.bytes, "ppt/charts/chart1.xml");
    assert!(
        chart.contains("Opening observation in the northern distribution and staging corridor")
    );
    assert!(chart.contains("<c:dispBlanksAs val=\"gap\"/>"));
    assert!(chart.contains("<c:tickLblPos val=\"none\"/>"));
    let workbook = part(&report.bytes, "ppt/embeddings/chart1.xlsx");
    let sheet = xml(&workbook, "xl/worksheets/sheet1.xml");
    assert!(
        sheet.contains("Opening observation in the northern distribution and staging corridor")
    );
    assert!(sheet.contains("<c r=\"B2\"><v>64</v></c>"));
    assert!(!sheet.contains("r=\"B3\""));
    assert!(sheet.contains("<c r=\"C3\"><v>63</v></c>"));
    let first_notes = xml(&report.bytes, "ppt/notesSlides/notesSlide1.xml");
    assert!(first_notes.contains("Definitions:"));
    assert!(first_notes.contains("Share of scheduled observations with a recorded result."));
    assert!(first_notes.contains("observation-log @ rev-4"));
    let chart_notes = xml(&report.bytes, "ppt/notesSlides/notesSlide2.xml");
    assert!(chart_notes.contains("Measured coverage: Not measured"));
    assert!(chart_notes.contains("Scheduled measurements and the recorded reference line."));
    assert!(chart_notes.matches("<a:p>").count() > 10);
}

#[test]
fn first_pdf_page_keeps_context_when_complete_identity_needs_more_footer_space() {
    let mut document = fixture();
    document.document_id = format!("{}-DOC-END", "W".repeat(246));
    document.revision = format!("{}-REV-END", "W".repeat(246));
    document.period = "2026-04-01 through 2026-06-30\nRecorded operating dates".into();
    let report = render_report(&document, &options(ReportFormat::Pdf)).unwrap();
    let pdf = lopdf::Document::load_mem(&report.bytes).unwrap();
    let first = pdf.extract_text(&[1]).unwrap();
    assert_in_order(&first, "Bottom line", "Period: 2026-04-01");
    assert!(first.contains("Comparison: Previous period"));
    let last = *pdf.get_pages().keys().last().unwrap();
    let provenance = pdf.extract_text(&[last]).unwrap();
    assert!(provenance.contains("-DOC-END"));
    assert!(provenance.contains("-REV-END"));
    let info = pdf.trailer.get(b"Info").unwrap().as_reference().unwrap();
    let subject = pdf
        .get_dictionary(info)
        .unwrap()
        .get(b"Subject")
        .unwrap()
        .as_str()
        .unwrap();
    let marker: Vec<u8> = [0xfe, 0xff]
        .into_iter()
        .chain(
            "office-report-binding-v1:"
                .encode_utf16()
                .flat_map(u16::to_be_bytes),
        )
        .collect();
    assert!(subject.starts_with(&marker));
    assert!(inspect_report(&report.bytes, &report.manifest).is_ok());
}

#[test]
fn dense_fixture_has_a_native_three_chart_grid_and_complete_linked_table_detail() {
    let document: ReportDocument = serde_json::from_str(DENSE).unwrap();
    let report = render_report(&document, &options(ReportFormat::Pptx)).unwrap();
    assert_eq!(report.manifest.pages_or_slides, Some(3));
    let first = xml(&report.bytes, "ppt/slides/slide1.xml");
    let second = xml(&report.bytes, "ppt/slides/slide2.xml");
    let third = xml(&report.bytes, "ppt/slides/slide3.xml");
    assert_in_order(
        &first,
        "Bottom line",
        "Recorded coverage reached 84 percent",
    );
    assert_in_order(&second, "Combined operation", "Staging workstream");
    assert_in_order(&second, "Staging workstream", "Dispatch workstream");
    assert!(first.contains("<p:txBody>"));
    let chart_frames = second.split("<p:graphicFrame>").skip(1).collect::<Vec<_>>();
    assert_eq!(chart_frames.len(), 3);
    let positions: Vec<u64> = chart_frames
        .iter()
        .map(|frame| {
            assert!(frame.contains("<c:chart "));
            let offset = frame.split_once("<a:off x=\"").unwrap().1;
            offset.split_once('"').unwrap().0.parse().unwrap()
        })
        .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    let relationships = xml(&report.bytes, "ppt/slides/_rels/slide2.xml.rels");
    for number in 1..=3 {
        assert!(relationships.contains(&format!("rIdChart{number}")));
        let chart = xml(&report.bytes, &format!("ppt/charts/chart{number}.xml"));
        assert!(chart.contains("<c:lineChart>"));
        assert!(chart.contains("dispBlanksAs val=\"gap\""));
        let workbook = part(&report.bytes, &format!("ppt/embeddings/chart{number}.xlsx"));
        let sheet = xml(&workbook, "xl/worksheets/sheet1.xml");
        assert!(
            sheet.contains("Opening checkpoint for the equivalent recorded operating schedule")
        );
        if number == 2 {
            assert!(sheet.contains("<c r=\"B2\"><v>67</v></c>"));
            assert!(!sheet.contains("r=\"B3\""));
            assert!(sheet.contains("<c r=\"B4\"><v>78</v></c>"));
        }
    }
    assert!(third.contains("<a:tbl>"));
    assert!(third.contains("<a:tc>"));
    assert!(third.contains("of 24 rows shown. Complete table in speaker notes."));
    let notes = xml(&report.bytes, "ppt/notesSlides/notesSlide3.xml");
    assert_eq!(notes.matches(">Row ").count(), 24);
    assert!(notes.contains("Row 24 (area-24)"));
    assert!(notes.contains("Area: Final dispatch staging corridor"));
    assert!(notes.contains("Recorded coverage: Not measured"));
    assert!(notes.contains("Target: 85"));
    let links = xml(&report.bytes, "ppt/slides/_rels/slide1.xml.rels");
    assert!(links.contains("Target=\"slide3.xml\""));
    assert!(links.contains("Target=\"slide2.xml\""));
    let core = xml(&report.bytes, "docProps/core.xml");
    assert!(core.contains("<cp:keywords>office-report-binding-v1:"));

    let pdf = render_report(&document, &options(ReportFormat::Pdf)).unwrap();
    let pdf = lopdf::Document::load_mem(&pdf.bytes).unwrap();
    let all = pdf
        .extract_text(&pdf.get_pages().into_keys().collect::<Vec<_>>())
        .unwrap();
    assert_in_order(&all, "Bottom line", "Report context");
    assert_in_order(&all, "Report context", "Combined coverage trends");
    assert_in_order(&all, "Combined coverage trends", "Coverage by work area");
    assert!(all.contains("Final dispatch staging corridor"));

    let html = render_report(&document, &options(ReportFormat::Html)).unwrap();
    let html = std::str::from_utf8(&html.bytes).unwrap();
    assert!(
        html.contains("<meta name=\"office-report-binding\" content=\"office-report-binding-v1:")
    );
    assert_in_order(html, "<h1>Bottom line</h1>", "<dt>Period</dt>");
    assert!(html.contains("<tr id=\"area-24\">"));
    assert!(html.contains("Final dispatch staging corridor"));
}

#[test]
fn compact_native_table_keeps_sixteen_rows_visible() {
    let mut document: ReportDocument = serde_json::from_str(DENSE).unwrap();
    let ReportBlock::AnchoredTable { table, row_ids } = &mut document.sections[2].blocks[0] else {
        unreachable!()
    };
    table.rows.truncate(16);
    row_ids.truncate(16);
    let ReportBlock::Links { links } = &mut document.sections[0].blocks[1] else {
        unreachable!()
    };
    links[0].target_id = "area-16".into();
    let report = render_report(&document, &options(ReportFormat::Pptx)).unwrap();
    let table = xml(&report.bytes, "ppt/slides/slide3.xml");
    assert_eq!(table.matches("<a:tr ").count(), 17);
    assert!(table.contains("Loading area 16"));
    assert!(!table.contains("Complete table in speaker notes"));
}

#[test]
fn paginated_appendix_retains_data_and_powerpoint_overflow_identifies_section() {
    let document: ReportDocument = serde_json::from_str(APPENDIX).unwrap();
    let deck = render_report(&document, &options(ReportFormat::Pptx)).unwrap();
    let notes = xml(&deck.bytes, "ppt/notesSlides/notesSlide2.xml");
    assert!(notes.contains("Row 40 (record:39)"));
    let mut overflow = document.clone();
    overflow.sections[1].blocks.push(ReportBlock::Paragraph {
        text: "This complete finding must stay visible within its section. ".repeat(260),
    });
    let error = render_report(&overflow, &options(ReportFormat::Pptx)).unwrap_err();
    assert!(matches!(error, ReportError::InvalidInput { path, message }
        if path == "sections" && message.contains("\"records\"") && message.contains("split its content")));
    let pdf = render_report(&document, &options(ReportFormat::Pdf)).unwrap();
    let parsed = lopdf::Document::load_mem(&pdf.bytes).unwrap();
    let pages = parsed.get_pages();
    let page_id = pages[&1];
    let annots = parsed
        .get_dictionary(page_id)
        .unwrap()
        .get(b"Annots")
        .unwrap()
        .as_array()
        .unwrap();
    let destinations = annots
        .iter()
        .map(|annotation| {
            let annotation = match annotation {
                lopdf::Object::Reference(id) => parsed.get_dictionary(*id).unwrap(),
                value => value.as_dict().unwrap(),
            };
            let action = annotation.get(b"A").unwrap().as_dict().unwrap();
            assert_eq!(action.get(b"S").unwrap().as_name().unwrap(), b"GoTo");
            let object = action.get(b"D").unwrap().as_array().unwrap()[0]
                .as_reference()
                .unwrap();
            let target = pages.iter().find(|&(_, id)| *id == object).unwrap().0;
            parsed.extract_text(&[*target]).unwrap()
        })
        .collect::<Vec<_>>();
    assert!(destinations.iter().any(|page| page.contains("Record 39")));

    let mut limited_html = options(ReportFormat::Html);
    limited_html.limits.max_pages = 1;
    let html = render_report(&document, &limited_html).unwrap();
    let text = std::str::from_utf8(&html.bytes).unwrap();
    assert!(text.contains("href=\"#record:39\""));
    assert!(text.contains("<tr id=\"record:39\">"));
    assert_eq!(html.manifest.pages_or_slides, None);

    let mut limited_slides = options(ReportFormat::Pptx);
    limited_slides.limits.max_pages = 2;
    let error = render_report(&fixture(), &limited_slides).unwrap_err();
    assert!(
        matches!(error, ReportError::LimitExceeded(message) if message.contains("\"paired-trends\""))
    );
}

#[test]
fn template_and_manifest_relabeling_fail_in_each_format() {
    let document = fixture();
    for format in FORMATS {
        let rendered = render_report(&document, &options(format)).unwrap();
        let old = render_report(&document, &ReportRenderOptions::new(format)).unwrap();
        for (artifact, template) in [
            (&rendered, REPORT_TEMPLATE),
            (&old, SECTION_FIRST_TEMPLATE),
            (&rendered, "unknown"),
        ] {
            let mut manifest = artifact.manifest.clone();
            manifest.template = template.into();
            assert!(inspect_report(&artifact.bytes, &manifest).is_err());
        }
        let edits: [fn(&mut ReportManifest); 7] = [
            |m: &mut ReportManifest| m.revision.push('x'),
            |m: &mut ReportManifest| m.document_id.push('x'),
            |m: &mut ReportManifest| m.evidence_sha256 = "c".repeat(64),
            |m: &mut ReportManifest| m.document_sha256 = "c".repeat(64),
            |m: &mut ReportManifest| m.sdk_version = "0.4.0".into(),
            |m: &mut ReportManifest| m.inspection.font_sha256 = "c".repeat(64),
            |m: &mut ReportManifest| m.pages_or_slides = Some(99),
        ];
        for edit in edits {
            let mut manifest = rendered.manifest.clone();
            edit(&mut manifest);
            assert!(inspect_report(&rendered.bytes, &manifest).is_err());
        }
        let mut bytes = rendered.bytes.clone();
        bytes[0] ^= 1;
        assert!(matches!(
            inspect_report(&bytes, &rendered.manifest),
            Err(ReportError::DigestMismatch)
        ));
    }
}

#[test]
fn inspection_flags_follow_actual_slide_content() {
    let mut document = fixture();
    document.sections.truncate(1);
    document.sections[0]
        .blocks
        .retain(|block| matches!(block, ReportBlock::Paragraph { .. }));
    let report = render_report(&document, &options(ReportFormat::Pptx)).unwrap();
    assert_eq!(report.manifest.pages_or_slides, Some(1));
    assert!(report.manifest.inspection.native_editable_text);
    assert!(!report.manifest.inspection.native_editable_tables);
    assert!(!report.manifest.inspection.native_editable_charts);
}

fn rebind_bytes(report: &RenderedReport, bytes: &[u8]) -> ReportManifest {
    let mut manifest = report.manifest.clone();
    manifest.byte_size = bytes.len();
    manifest.output_sha256 = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    manifest
}

fn replace_part(bytes: &[u8], path: &str, from: &str, to: &str) -> Vec<u8> {
    let mut input = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut output = ZipWriter::new(Cursor::new(Vec::new()));
    let mut replaced = false;
    for index in 0..input.len() {
        let mut entry = input.by_index(index).unwrap();
        let name = entry.name().unwrap().into_owned();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if name == path {
            let xml = String::from_utf8(bytes).unwrap();
            assert!(xml.contains(from));
            bytes = xml.replacen(from, to, 1).into_bytes();
            replaced = true;
        }
        output
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    assert!(replaced);
    output.finish().unwrap().into_inner()
}

#[test]
fn rehashed_artifacts_must_still_satisfy_section_first_structure() {
    let document = fixture();
    let pptx = render_report(&document, &options(ReportFormat::Pptx)).unwrap();
    for (path, from, to) in [
        ("ppt/presentation.xml", "r:id=\"rId2\"", "r:id=\"rId3\""),
        (
            "ppt/slides/slide3.xml",
            "r:id=\"rIdChart2\"",
            "r:id=\"rIdChart3\"",
        ),
        (
            "ppt/charts/chart1.xml",
            "dispBlanksAs val=\"gap\"",
            "dispBlanksAs val=\"zero\"",
        ),
        (
            "ppt/charts/_rels/chart1.xml.rels",
            "embeddings/chart1.xlsx",
            "embeddings/chart2.xlsx",
        ),
        (
            "ppt/slides/slide1.xml",
            "r:id=\"rIdNavigation",
            "r:id=\"missing",
        ),
    ] {
        let bytes = replace_part(&pptx.bytes, path, from, to);
        assert!(inspect_report(&bytes, &rebind_bytes(&pptx, &bytes)).is_err());
    }
    let html = render_report(&document, &options(ReportFormat::Html)).unwrap();
    let text = std::str::from_utf8(&html.bytes).unwrap();
    for altered in [
        text.replacen("name=\"office-report-binding\"", "name=\"unknown\"", 1),
        text.replacen("<main><section", "<main><p>Added preamble</p><section", 1),
    ] {
        let bytes = altered.as_bytes();
        assert!(inspect_report(bytes, &rebind_bytes(&html, bytes)).is_err());
    }
}

#[test]
fn opening_chart_uses_a_second_level_heading_only_in_section_first_html() {
    let mut document = fixture();
    let chart = document
        .sections
        .iter()
        .flat_map(|section| &section.blocks)
        .find(|block| matches!(block, ReportBlock::Chart { .. }))
        .unwrap()
        .clone();
    let ReportBlock::Chart { chart } = chart else {
        unreachable!()
    };
    let mut opening = chart.clone();
    opening.title = "Opening observations".into();
    document.sections[0]
        .blocks
        .insert(0, ReportBlock::Chart { chart: opening });
    let candidate = render_report(&document, &options(ReportFormat::Html)).unwrap();
    let output = std::str::from_utf8(&candidate.bytes).unwrap();
    assert!(output.contains("<figcaption><h2>Opening observations</h2>"));
    assert!(output.contains(&format!("<figcaption><h3>{}</h3>", chart.title)));
    let skipped = output.replacen(
        "<h2>Opening observations</h2>",
        "<h3>Opening observations</h3>",
        1,
    );
    assert_invalid_artifact(
        &candidate,
        skipped.as_bytes(),
        "incorrect chart heading level",
    );

    let old = render_report(&document, &ReportRenderOptions::new(ReportFormat::Html)).unwrap();
    assert!(
        std::str::from_utf8(&old.bytes)
            .unwrap()
            .contains("<figcaption><h3>Opening observations</h3>")
    );
}

#[test]
fn section_first_html_requires_an_opening_section_directly_inside_main() {
    let report = render_report(&fixture(), &options(ReportFormat::Html)).unwrap();
    let output = std::str::from_utf8(&report.bytes).unwrap();
    assert!(output.contains("<main><section id=\"bottom-line\"><h1>Bottom line</h1>"));
    assert!(inspect_report(&report.bytes, &report.manifest).is_ok());
    let escaped =
        output
            .replacen("<main>", "<main></main>", 1)
            .replacen("</footer></main>", "</footer>", 1);
    assert_invalid_artifact(
        &report,
        escaped.as_bytes(),
        "first reading section is missing",
    );
    let prior = output.replacen("<main>", "<main>&lt;section&gt;", 1);
    assert_invalid_artifact(
        &report,
        prior.as_bytes(),
        "text precedes the first reading section",
    );
    let misplaced = output.replacen("<main>", "<main><div>", 1).replacen(
        "</footer></main>",
        "</footer></div></main>",
        1,
    );
    assert_invalid_artifact(
        &report,
        misplaced.as_bytes(),
        "first reading section is missing",
    );
    let outside = output.replacen("</main>", "</main><section><h2>Outside</h2></section>", 1);
    assert_invalid_artifact(
        &report,
        outside.as_bytes(),
        "report section is outside main",
    );
}

#[test]
fn native_heading_must_be_meaningful_text_in_the_heading_textbox() {
    let document: ReportDocument = serde_json::from_str(DENSE).unwrap();
    let report = render_report(&document, &options(ReportFormat::Pptx)).unwrap();
    let path = "ppt/slides/slide3.xml";
    let slide = xml(&report.bytes, path);
    assert!(slide.contains("id=\"2\" name=\"Report text 2\""));
    assert!(slide.contains("<a:tbl>"));
    assert!(
        inspect_report(&report.bytes, &report.manifest)
            .unwrap()
            .native_editable_text
    );
    for blank in ["", "   ", "&#x200b;"] {
        let bytes = replace_part(
            &report.bytes,
            path,
            ">Coverage by work area</a:t>",
            &format!(">{blank}</a:t>"),
        );
        assert_invalid_artifact(&report, &bytes, "section slide has no editable heading");
    }
    let bytes = replace_part(
        &report.bytes,
        path,
        r#"<p:cNvSpPr txBox="1"/>"#,
        "<p:cNvSpPr/>",
    );
    assert_invalid_artifact(&report, &bytes, "section slide has no editable heading");
}

#[test]
fn preview_caption_is_valid_prose_with_or_without_tables() {
    const CAPTION: &str = "6 of 24 rows shown. Complete table in speaker notes.";
    let paragraph = || ReportBlock::Paragraph {
        text: CAPTION.into(),
    };
    let table = |rows| ReportBlock::Table {
        table: ReportTable {
            caption: String::new(),
            headers: vec!["Observation".into()],
            rows: (1..=rows)
                .map(|row| vec![format!("Recorded result {row}")])
                .collect(),
        },
    };
    let mut document = fixture();
    document.sections = vec![
        ReportSection {
            id: "prose".into(),
            title: "Report prose".into(),
            blocks: vec![paragraph()],
        },
        ReportSection {
            id: "mixed".into(),
            title: "Recorded results".into(),
            blocks: vec![table(1), paragraph(), table(24), paragraph()],
        },
    ];
    let report = render_report(&document, &options(ReportFormat::Pptx)).unwrap();
    let inspection = inspect_report(&report.bytes, &report.manifest).unwrap();
    assert!(inspection.native_editable_tables);

    let prose = xml(&report.bytes, "ppt/slides/slide1.xml");
    assert!(prose.contains(CAPTION));
    assert!(!prose.contains("office-report-table-preview-v1:"));

    let mixed = xml(&report.bytes, "ppt/slides/slide2.xml");
    let captions = mixed
        .split("<p:sp>")
        .skip(1)
        .map(|shape| shape.split_once("</p:sp>").unwrap().0)
        .filter(|shape| shape.contains(CAPTION))
        .collect::<Vec<_>>();
    let [before, preview, after] = captions.as_slice() else {
        panic!("expected two ordinary paragraphs and one generated preview");
    };
    for prose in [before, after] {
        assert!(prose.contains(r#"name="Report text "#));
        assert!(!prose.contains(" descr="));
    }
    assert!(
        preview
            .contains(r#"name="Report table preview" descr="office-report-table-preview-v1:6:24:"#)
    );
    let notes = xml(&report.bytes, "ppt/notesSlides/notesSlide2.xml");
    assert_eq!(notes.matches("Complete table: ").count(), 1);
    assert!(notes.contains("Observation: Recorded result 24"));
}

#[test]
fn preview_requires_the_complete_record_in_the_native_notes_body() {
    let mut document: ReportDocument = serde_json::from_str(DENSE).unwrap();
    let ReportBlock::AnchoredTable { table, .. } = &mut document.sections[2].blocks[0] else {
        unreachable!()
    };
    table.caption = "Recorded work areas & measurements\r\nComplete selection".into();
    table.rows[23][0] = "Final dispatch & staging corridor\r\nExtended detail".into();
    let report = render_report(&document, &options(ReportFormat::Pptx)).unwrap();
    assert!(inspect_report(&report.bytes, &report.manifest).is_ok());
    let path = "ppt/notesSlides/notesSlide3.xml";
    let body = xml(&report.bytes, path);
    assert!(body.contains("Final dispatch &amp; staging corridor"));
    let wrong_placeholder = replace_part(
        &report.bytes,
        path,
        r#"p:ph type="body" idx="1""#,
        r#"p:ph type="ctrTitle" idx="1""#,
    );
    assert_invalid_artifact(
        &report,
        &wrong_placeholder,
        "table preview requires a unique notes body",
    );

    let start = body.find("<a:p>").unwrap();
    let end = body.rfind("</a:p>").unwrap() + "</a:p>".len();
    let empty_notes = replace_part(&report.bytes, path, &body[start..end], "<a:p/>");
    assert_invalid_artifact(
        &report,
        &empty_notes,
        "table preview does not match its complete table",
    );
    let missing_value = replace_part(
        &report.bytes,
        path,
        "Area: Final dispatch &amp; staging corridor",
        "Area: ",
    );
    assert_invalid_artifact(
        &report,
        &missing_value,
        "table preview does not match its complete table",
    );
    let slide_path = "ppt/slides/slide3.xml";
    let tampered_marker = replace_part(
        &report.bytes,
        slide_path,
        "office-report-table-preview-v1:",
        "unused-preview-marker:",
    );
    assert_invalid_artifact(
        &report,
        &tampered_marker,
        "table preview does not match its complete table",
    );

    let slide = xml(&report.bytes, slide_path);
    let name = r#"name="Report table preview""#;
    let (_, description) = slide.split_once(name).unwrap();
    let (attribute, _) = description.split_once("/>").unwrap();
    assert!(attribute.starts_with(r#" descr="office-report-table-preview-v1:"#));
    let missing_marker = replace_part(
        &report.bytes,
        slide_path,
        &format!("{name}{attribute}"),
        name,
    );
    assert_invalid_artifact(
        &report,
        &missing_marker,
        "table preview does not match its complete table",
    );
    let revised_caption = replace_part(
        &missing_marker,
        slide_path,
        "rows shown. Complete table in speaker notes.",
        "recorded rows",
    );
    assert_invalid_artifact(
        &report,
        &revised_caption,
        "table preview does not match its complete table",
    );
}

#[test]
fn section_chart_requires_the_chart_plot_and_line_hierarchy() {
    let report = render_report(&fixture(), &options(ReportFormat::Pptx)).unwrap();
    let path = "ppt/charts/chart1.xml";
    let chart = xml(&report.bytes, path);
    assert!(chart.contains("<c:plotArea><c:layout/><c:lineChart>"));
    assert!(
        inspect_report(&report.bytes, &report.manifest)
            .unwrap()
            .native_editable_charts
    );

    let outside_plot = replace_part(&report.bytes, path, "</c:valAx></c:plotArea>", "</c:valAx>");
    let outside_plot = replace_part(
        &outside_plot,
        path,
        "<c:plotArea><c:layout/>",
        "<c:plotArea><c:layout/></c:plotArea>",
    );
    assert_invalid_artifact(
        &report,
        &outside_plot,
        "unexpected native chart element position",
    );
    let outside_chart = replace_part(
        &report.bytes,
        path,
        r#"<c:dispBlanksAs val="gap"/></c:chart>"#,
        r#"</c:chart><c:dispBlanksAs val="gap"/>"#,
    );
    assert_invalid_artifact(
        &report,
        &outside_chart,
        "unexpected native chart element position",
    );
    let nested_workbook = replace_part(
        &report.bytes,
        path,
        "</c:chart><c:externalData",
        "<c:externalData",
    );
    let nested_workbook = replace_part(
        &nested_workbook,
        path,
        "</c:externalData></c:chartSpace>",
        "</c:externalData></c:chart></c:chartSpace>",
    );
    assert_invalid_artifact(
        &report,
        &nested_workbook,
        "unexpected native chart element position",
    );
    let foreign_plot = replace_part(
        &report.bytes,
        path,
        "<c:plotArea>",
        r#"<c:plotArea xmlns:c="urn:other">"#,
    );
    assert_invalid_artifact(&report, &foreign_plot, "unexpected OOXML element namespace");
}

#[test]
fn long_pdf_context_preserves_a_labeled_preview_and_unabridged_continuation() {
    for long_period in [true, false] {
        let mut document = fixture();
        let long_value = [
            "Anchor for this selected interval",
            "Recorded interval alpha",
            "Recorded interval beta",
            "Recorded interval gamma",
            "Recorded interval delta",
            "Ending reference interval",
        ]
        .join("\n");
        let (long_label, ordinary) = if long_period {
            document.period = long_value.clone();
            document.comparison = "Ordinary comparison\nwith equivalent scope".into();
            ("Period", "with equivalent scope")
        } else {
            document.comparison = long_value.clone();
            document.period = "Ordinary period\nwith a complete date".into();
            ("Comparison", "with a complete date")
        };
        let report = render_report(&document, &options(ReportFormat::Pdf)).unwrap();
        let pdf = lopdf::Document::load_mem(&report.bytes).unwrap();
        let first = pdf.extract_text(&[1]).unwrap();
        assert!(first.contains(&format!(
            "{long_label}: Anchor for this selected interval ..."
        )));
        assert!(first.contains(ordinary));
        assert!(first.contains("Full values continue in Report context."));
        assert!(!first.contains("Ending reference interval"));
        let context = pdf
            .get_pages()
            .into_keys()
            .skip(1)
            .map(|page| pdf.extract_text(&[page]).unwrap())
            .find(|page| page.contains("Report context"))
            .unwrap();
        assert_in_order(
            &context,
            &format!("{long_label}: Anchor for this selected interval"),
            "Ending reference interval",
        );
        for line in long_value.lines() {
            assert!(context.contains(line), "context lost {line}");
        }
    }
    let ordinary = render_report(&fixture(), &options(ReportFormat::Pdf)).unwrap();
    let first = lopdf::Document::load_mem(&ordinary.bytes)
        .unwrap()
        .extract_text(&[1])
        .unwrap();
    assert!(first.contains("Period: April to June 2026"));
    assert!(first.contains("Comparison: Previous period, equivalent scope"));
    assert!(!first.contains("Full values continue"));
}

#[test]
fn compact_slide_fits_a_412_point_table_and_previews_only_after_overflow() {
    let mut document = fixture();
    document.sections = vec![ReportSection {
        id: "measured".into(),
        title: "Recorded results".into(),
        blocks: vec![ReportBlock::Table {
            table: ReportTable {
                caption: String::new(),
                headers: vec!["Observation".into()],
                rows: (1..=18)
                    .map(|row| vec![format!("Recorded result {row}")])
                    .collect(),
            },
        }],
    }];
    let ReportBlock::Table { table } = &mut document.sections[0].blocks[0] else {
        unreachable!()
    };
    table.rows[0][0].push_str("\nfirst measured detail");
    let report = render_report(&document, &options(ReportFormat::Pptx)).unwrap();
    let slide = xml(&report.bytes, "ppt/slides/slide1.xml");
    assert_eq!(slide.matches("<a:tr ").count(), 19);
    assert!(slide.contains("Recorded result 18"));
    assert!(!slide.contains("Complete table in speaker notes."));

    let ReportBlock::Table { table } = &mut document.sections[0].blocks[0] else {
        unreachable!()
    };
    table.rows[1][0].push_str("\nsecond measured detail");
    let report = render_report(&document, &options(ReportFormat::Pptx)).unwrap();
    let slide = xml(&report.bytes, "ppt/slides/slide1.xml");
    assert!(slide.contains("of 18 rows shown. Complete table in speaker notes."));
    let notes = xml(&report.bytes, "ppt/notesSlides/notesSlide1.xml");
    assert!(notes.contains("Observation: Recorded result 18"));
}

fn assert_invalid_artifact(report: &RenderedReport, bytes: &[u8], expected: &str) {
    let error = inspect_report(bytes, &rebind_bytes(report, bytes)).unwrap_err();
    assert!(error.to_string().contains(expected), "{error}");
}

#[test]
#[ignore = "requires artifacts generated from the pinned previous SDK in CI"]
fn previous_release_artifacts_remain_readable() {
    let output = std::path::PathBuf::from(std::env::var_os("OFFICE_REPORT_BASELINE_DIR").unwrap());
    let document = fixture();
    for format in FORMATS {
        let extension = format.extension();
        let bytes = std::fs::read(output.join(format!("report.{extension}"))).unwrap();
        let manifest: ReportManifest = serde_json::from_slice(
            &std::fs::read(output.join(format!("report.{extension}.manifest.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(manifest.sdk_version, "0.4.0");
        assert_eq!(manifest.template, REPORT_TEMPLATE);
        assert_eq!(manifest.format, format);
        assert_eq!(
            manifest.document_sha256,
            report_document_sha256(&document).unwrap()
        );
        let inspection = inspect_report(&bytes, &manifest).unwrap();
        assert_eq!(
            inspection.structural_checks,
            manifest.inspection.structural_checks
        );
        assert_eq!(
            inspection.native_editable_charts,
            format == ReportFormat::Pptx
        );
        assert!(inspection.visual_review_required);
    }
}
