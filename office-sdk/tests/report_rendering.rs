use office_sdk::{
    OfficeDocument,
    pptx::{PptxDocument, SlideSize},
    report::*,
};
use std::io::{Cursor, Read};
use zip::ZipArchive;
fn fixture() -> ReportDocument {
    serde_json::from_str(include_str!("fixtures/executive-report.json")).unwrap()
}
fn zip_part(bytes: &[u8], name: &str) -> String {
    let mut z = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut s = String::new();
    z.by_name(name).unwrap().read_to_string(&mut s).unwrap();
    s
}
fn render(d: &ReportDocument, f: ReportFormat) -> RenderedReport {
    render_report(d, &ReportRenderOptions::new(f)).unwrap()
}

#[test]
fn same_document_binds_all_formats_and_preserves_facts() {
    let d = fixture();
    let digest = report_document_sha256(&d).unwrap();
    for f in [ReportFormat::Pdf, ReportFormat::Html, ReportFormat::Pptx] {
        let r = render(&d, f);
        assert_eq!(r.manifest.document_sha256, digest);
        assert_eq!(r.manifest.evidence_sha256, d.provenance.evidence_sha256);
        assert_eq!(r.manifest.byte_size, r.bytes.len());
        assert!(r.manifest.inspection.visual_review_required);
        match f {
            ReportFormat::Pdf => {
                let pdf = lopdf::Document::load_mem(&r.bytes).unwrap();
                assert_eq!(Some(pdf.get_pages().len()), r.manifest.pages_or_slides);
                let pages = (1..=pdf.get_pages().len() as u32).collect::<Vec<_>>();
                let text = pdf.extract_text(&pages).unwrap();
                for value in [
                    "64",
                    "75",
                    "Production",
                    "Développement",
                    "Защита",
                    "Not measured",
                    "Previous period",
                    "Definitions",
                ] {
                    assert!(text.contains(value), "PDF lost {value}: {text}");
                }
            }
            ReportFormat::Html => {
                let text = String::from_utf8(r.bytes).unwrap();
                for value in [
                    "64",
                    "75",
                    "Développement",
                    "Защита",
                    "Not measured",
                    "Previous period",
                ] {
                    assert!(text.contains(value));
                }
            }
            ReportFormat::Pptx => {
                let p = zip_part(&r.bytes, "ppt/presentation.xml");
                assert!(p.contains("cx=\"12192000\" cy=\"6858000\""));
                assert!(
                    zip_part(&r.bytes, "ppt/notesSlides/notesSlide1.xml").contains("Definitions:")
                );
            }
        }
    }
}

#[test]
fn pptx_contains_native_tables_charts_editable_workbook_and_gap_semantics() {
    let r = render(&fixture(), ReportFormat::Pptx);
    let mut z = ZipArchive::new(Cursor::new(&r.bytes)).unwrap();
    let names = (0..z.len())
        .map(|i| z.by_index(i).unwrap().name().unwrap().into_owned())
        .collect::<Vec<_>>();
    let chart_name = names
        .iter()
        .find(|n| n.starts_with("ppt/charts/chart") && n.ends_with(".xml"))
        .unwrap();
    let chart = zip_part(&r.bytes, chart_name);
    assert!(chart.contains("<c:lineChart>"));
    assert!(chart.contains("<c:dispBlanksAs val=\"gap\"/>"));
    assert!(chart.contains("<c:externalData r:id=\"rIdWorkbook\""));
    assert!(chart.contains("Sheet1!$B$2:$B$5"));
    assert!(!chart.contains("<c:pt idx=\"1\"><c:v>0</c:v>"));
    let workbook_name = names
        .iter()
        .find(|n| n.starts_with("ppt/embeddings/"))
        .unwrap();
    let mut workbook = Vec::new();
    z.by_name(workbook_name)
        .unwrap()
        .read_to_end(&mut workbook)
        .unwrap();
    let sheet = zip_part(&workbook, "xl/worksheets/sheet1.xml");
    assert!(sheet.contains("<c r=\"B2\"><v>61</v></c>"));
    assert!(!sheet.contains("r=\"B3\""));
    assert!(sheet.contains("<c r=\"B5\"><v>64</v></c>"));
    assert!(
        names
            .iter()
            .filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml"))
            .any(|n| zip_part(&r.bytes, n).contains("<a:tbl>"))
    );
    // Demonstrate an editable text/chart-data round trip using real XML parts,
    // not screenshot images: the payload is native text and numeric cells.
    assert!(names.iter().all(|n| !n.starts_with("ppt/media/")));
}

#[test]
fn html_escapes_injection_and_has_offline_accessible_data() {
    let mut d = fixture();
    d.title = "<script>alert('x')</script> & report".into();
    d.sections[0].blocks = vec![ReportBlock::Paragraph {
        text: "<img src=https://example.com onerror=alert(1)> \" &".into(),
    }];
    let h = String::from_utf8(render(&d, ReportFormat::Html).bytes).unwrap();
    assert!(!h.contains("<script>"));
    assert!(!h.contains("<img src="));
    assert!(h.contains("&lt;script&gt;"));
    assert!(h.contains("Content-Security-Policy"));
    assert!(h.contains("scope=\"col\""));
    assert!(h.contains("<desc>"));
    assert!(h.contains("Not measured"));
}

#[test]
fn limits_hash_schema_and_nonfinite_data_fail_closed() {
    let d = fixture();
    let mut o = ReportRenderOptions::new(ReportFormat::Pdf);
    o.expected_document_sha256 = Some("b".repeat(64));
    assert!(matches!(
        render_report(&d, &o),
        Err(ReportError::DigestMismatch)
    ));
    o.expected_document_sha256 = None;
    o.limits.max_pages = 1;
    assert!(matches!(
        render_report(&d, &o),
        Err(ReportError::LimitExceeded(_))
    ));
    let mut d = fixture();
    d.schema_version = 2;
    assert!(render_report(&d, &ReportRenderOptions::new(ReportFormat::Html)).is_err());
    let mut d = fixture();
    if let ReportBlock::Chart { chart } = &mut d.sections[1].blocks[1] {
        chart.series[0].values[0] = Some(f64::NAN);
    }
    assert!(render_report(&d, &ReportRenderOptions::new(ReportFormat::Pptx)).is_err());
    let mut d = fixture();
    d.sections[0].blocks.push(ReportBlock::Paragraph {
        text: "illegal\0text".into(),
    });
    assert!(render_report(&d, &ReportRenderOptions::new(ReportFormat::Pdf)).is_err());
}

#[test]
fn long_rows_paginate_without_losing_names_or_last_row() {
    let mut d = fixture();
    let rows = (0..80)
        .map(|i| {
            vec![
                format!("{i}: {}", "full-environment-name-".repeat(12)),
                format!("last-value-{i}"),
            ]
        })
        .collect();
    d.sections = vec![ReportSection {
        id: "large".into(),
        title: "Large table".into(),
        blocks: vec![ReportBlock::Table {
            table: ReportTable {
                caption: "All rows".into(),
                headers: vec!["Environment".into(), "Fact".into()],
                rows,
            },
        }],
    }];
    let r = render(&d, ReportFormat::Pdf);
    let pdf = lopdf::Document::load_mem(&r.bytes).unwrap();
    assert!(pdf.get_pages().len() > 10);
    let text = pdf
        .extract_text(&(1..=pdf.get_pages().len() as u32).collect::<Vec<_>>())
        .unwrap();
    assert!(text.contains("last-value-79"));
}

#[test]
fn empty_document_and_missing_glyphs_are_explicit() {
    let mut d = fixture();
    d.sections.clear();
    d.definitions.clear();
    d.limitations.clear();
    assert!(
        render(&d, ReportFormat::Pdf)
            .manifest
            .pages_or_slides
            .unwrap()
            >= 2
    );
    d.title = "Unsupported 🦀 glyph".into();
    assert!(matches!(
        render_report(&d, &ReportRenderOptions::new(ReportFormat::Pdf)),
        Err(ReportError::UnsupportedGlyph(_))
    ));
}

#[test]
fn legacy_slide_size_is_serialized_before_and_after_adding_slides() {
    for (size, w, h) in [
        (SlideSize::Widescreen16x9, 12192000, 6858000),
        (SlideSize::Standard4x3, 9144000, 6858000),
        (
            SlideSize::Custom {
                width: 10000000,
                height: 5000000,
            },
            10000000,
            5000000,
        ),
    ] {
        let mut p = PptxDocument::new();
        p.set_slide_size(size);
        assert!(
            zip_part(&p.to_bytes().unwrap(), "ppt/presentation.xml")
                .contains(&format!("cx=\"{w}\" cy=\"{h}\""))
        );
        p.add_slide("Title", &["Body"]).unwrap();
        assert!(
            zip_part(&p.to_bytes().unwrap(), "ppt/presentation.xml")
                .contains(&format!("cx=\"{w}\" cy=\"{h}\""))
        );
    }
}

#[test]
fn inspection_rejects_changed_bytes_and_forged_counts() {
    let d = fixture();
    let r = render(&d, ReportFormat::Pptx);
    let mut modified = r.bytes.clone();
    modified[10] ^= 1;
    assert!(matches!(
        inspect_report(&modified, &r.manifest),
        Err(ReportError::DigestMismatch)
    ));
    let mut manifest = r.manifest.clone();
    manifest.pages_or_slides = Some(999);
    assert!(inspect_report(&r.bytes, &manifest).is_err());
    let mut manifest = r.manifest.clone();
    manifest.mime_type = "application/pdf".into();
    assert!(inspect_report(&r.bytes, &manifest).is_err());
}

#[test]
fn imported_standard_and_custom_dimensions_survive_new_slides() {
    for size in [
        SlideSize::Standard4x3,
        SlideSize::Custom {
            width: 10000000,
            height: 5000000,
        },
    ] {
        let mut p = PptxDocument::new();
        p.set_slide_size(size);
        p.add_slide("Original", &[]).unwrap();
        let mut loaded = PptxDocument::from_bytes(&p.to_bytes().unwrap()).unwrap();
        assert_eq!(loaded.get_slide_size(), size);
        loaded.add_slide("Added", &[]).unwrap();
        let (w, h) = size.dimensions();
        assert!(
            zip_part(&loaded.to_bytes().unwrap(), "ppt/presentation.xml")
                .contains(&format!("cx=\"{w}\" cy=\"{h}\""))
        );
    }
}

fn rewrite_zip(bytes: &[u8], mut replace: impl FnMut(&str, Vec<u8>) -> Vec<u8>) -> Vec<u8> {
    use std::io::Write;
    let mut source = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut target = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..source.len() {
        let mut entry = source.by_index(i).unwrap();
        let name = entry.name().unwrap().into_owned();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).unwrap();
        target
            .start_file(
                &name,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated),
            )
            .unwrap();
        target.write_all(&replace(&name, data)).unwrap();
    }
    target.finish().unwrap().into_inner()
}
fn rebind_bytes(mut rendered: RenderedReport, bytes: Vec<u8>) -> RenderedReport {
    use sha2::{Digest, Sha256};
    rendered.manifest.byte_size = bytes.len();
    rendered.manifest.output_sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    rendered.bytes = bytes;
    rendered
}

#[test]
fn inspection_rejects_incomplete_powerpoint_theme_style_lists() {
    for (list, one_style) in [
        (
            "fillStyleLst",
            "<a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill>",
        ),
        (
            "lnStyleLst",
            "<a:ln><a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill></a:ln>",
        ),
        (
            "effectStyleLst",
            "<a:effectStyle><a:effectLst/></a:effectStyle>",
        ),
        (
            "bgFillStyleLst",
            "<a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill>",
        ),
    ] {
        let rendered = render(&fixture(), ReportFormat::Pptx);
        let bytes = rewrite_zip(&rendered.bytes, |name, bytes| {
            if name != "ppt/theme/theme1.xml" {
                return bytes;
            }
            let mut theme = String::from_utf8(bytes).unwrap();
            let opening = format!("<a:{list}>");
            let start = theme.find(&opening).unwrap() + opening.len();
            let end = start + theme[start..].find(&format!("</a:{list}>")).unwrap();
            theme.replace_range(start..end, one_style);
            theme.into_bytes()
        });
        let modified = rebind_bytes(rendered, bytes);
        assert!(
            inspect_report(&modified.bytes, &modified.manifest).is_err(),
            "PowerPoint requires three styles in {list}, even when the package hash matches"
        );
    }
}

#[test]
fn inspection_resolves_theme_namespace_aliases() {
    let rendered = render(&fixture(), ReportFormat::Pptx);
    let bytes = rewrite_zip(&rendered.bytes, |name, bytes| {
        if name != "ppt/theme/theme1.xml" {
            return bytes;
        }
        String::from_utf8(bytes)
            .unwrap()
            .replace("xmlns:a=", "xmlns:d=")
            .replace("a:", "d:")
            .into_bytes()
    });
    let modified = rebind_bytes(rendered, bytes);
    assert!(inspect_report(&modified.bytes, &modified.manifest).is_ok());
}

#[test]
fn inspection_rejects_theme_style_children_in_an_unrelated_namespace() {
    let rendered = render(&fixture(), ReportFormat::Pptx);
    let bytes = rewrite_zip(&rendered.bytes, |name, bytes| {
        if name != "ppt/theme/theme1.xml" {
            return bytes;
        }
        String::from_utf8(bytes)
            .unwrap()
            .replace("<a:ln w=", "<a:ln xmlns:a=\"urn:unrelated\" w=")
            .into_bytes()
    });
    let modified = rebind_bytes(rendered, bytes);
    assert!(inspect_report(&modified.bytes, &modified.manifest).is_err());
}

#[test]
fn inspection_rejects_nested_workbook_expansion() {
    let rendered = render(&fixture(), ReportFormat::Pptx);
    let bytes = rewrite_zip(&rendered.bytes, |name, data| {
        if !name.ends_with(".xlsx") {
            return data;
        }
        rewrite_zip(&data, |inner, original| {
            if inner != "xl/worksheets/sheet1.xml" {
                return original;
            }
            let mut expanded = b"<worksheet>".to_vec();
            expanded.resize(129 * 1024 * 1024, b' ');
            expanded.extend_from_slice(b"</worksheet>");
            expanded
        })
    });
    let modified = rebind_bytes(rendered, bytes);
    assert!(
        matches!(
            inspect_report(&modified.bytes, &modified.manifest),
            Err(ReportError::LimitExceeded(_))
        ),
        "embedded workbook expansion must share the bounded archive budget"
    );
}
#[test]
fn inspection_rejects_active_html_with_mixed_case_tags() {
    let rendered = render(&fixture(), ReportFormat::Html);
    let original = String::from_utf8(rendered.bytes.clone()).unwrap();
    let bytes = original
        .replace("</main>", "<ScRiPt>alert(1)</ScRiPt></main>")
        .into_bytes();
    let modified = rebind_bytes(rendered, bytes);
    assert!(
        inspect_report(&modified.bytes, &modified.manifest).is_err(),
        "HTML active-content inspection must follow HTML's case-insensitive tag semantics"
    );
}

#[test]
fn inspection_rejects_active_html_attributes_and_fake_policy() {
    let rendered = render(&fixture(), ReportFormat::Html);
    let original = String::from_utf8(rendered.bytes.clone()).unwrap();
    for addition in [
        "<svg onload=\"alert(1)\"></svg>",
        "<meta http-equiv=\"refresh\" content=\"0;url=https://example.com\">",
    ] {
        let bytes = original
            .replace("</main>", &format!("{addition}</main>"))
            .into_bytes();
        let modified = rebind_bytes(rendered.clone(), bytes);
        assert!(inspect_report(&modified.bytes, &modified.manifest).is_err());
    }
    let start = original
        .find("<meta http-equiv=\"Content-Security-Policy\"")
        .unwrap();
    let end = start + original[start..].find('>').unwrap() + 1;
    let mut fake = original;
    fake.replace_range(start..end, "<!-- Content-Security-Policy -->");
    let modified = rebind_bytes(rendered, fake.into_bytes());
    assert!(inspect_report(&modified.bytes, &modified.manifest).is_err());
}
#[test]
fn inspection_rejects_nested_workbook_external_relationships() {
    let rendered = render(&fixture(), ReportFormat::Pptx);
    let bytes = rewrite_zip(&rendered.bytes, |name, data| {
        if !name.ends_with(".xlsx") {
            return data;
        }
        rewrite_zip(&data, |inner, original| {
            if inner != "xl/_rels/workbook.xml.rels" {
                return original;
            }
            String::from_utf8(original).unwrap().replace("</Relationships>", "<Relationship Id=\"external\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/externalLink\" Target=\"https://example.com/data.xlsx\" TargetMode=\"External\"/></Relationships>").into_bytes()
        })
    });
    let modified = rebind_bytes(rendered, bytes);
    assert!(inspect_report(&modified.bytes, &modified.manifest).is_err());
}

#[test]
fn inspection_rejects_html_parser_comment_differentials() {
    let rendered = render(&fixture(), ReportFormat::Html);
    let original = String::from_utf8(rendered.bytes.clone()).unwrap();
    for addition in [
        "<!-- --!><meta http-equiv=\"refresh\" content=\"0;url=https://example.com\"> -->",
        "<!--><meta http-equiv=\"refresh\" content=\"0;url=https://example.com\"> -->",
        "<![CDATA[<meta http-equiv=\"refresh\" content=\"0;url=https://example.com\">]]>",
    ] {
        let bytes = original
            .replace("</main>", &format!("{addition}</main>"))
            .into_bytes();
        let modified = rebind_bytes(rendered.clone(), bytes);
        assert!(
            inspect_report(&modified.bytes, &modified.manifest).is_err(),
            "accepted {addition}"
        );
    }
}

#[test]
fn long_identity_footers_fail_instead_of_clipping() {
    let mut d = fixture();
    d.document_id = "W".repeat(256);
    d.revision = "W".repeat(256);
    validate_report_document(&d, &ReportLimits::default()).unwrap();
    for format in [ReportFormat::Pdf, ReportFormat::Pptx] {
        assert!(
            render_report(&d, &ReportRenderOptions::new(format)).is_err(),
            "accepted oversized {format:?} footer"
        );
    }
}

#[test]
fn pdf_chart_identifies_each_series_on_the_chart_page() {
    let mut d = fixture();
    let chart = d
        .sections
        .iter_mut()
        .flat_map(|s| &mut s.blocks)
        .find_map(|b| match b {
            ReportBlock::Chart { chart } => Some(chart),
            _ => None,
        })
        .unwrap();
    let mut other = chart.series[0].clone();
    other.name = "Prior readiness".into();
    chart.series.push(other);
    let r = render(&d, ReportFormat::Pdf);
    let pdf = lopdf::Document::load_mem(&r.bytes).unwrap();
    let chart_page = (1..=pdf.get_pages().len() as u32)
        .map(|p| pdf.extract_text(&[p]).unwrap())
        .find(|text| text.contains("following chart-data table"))
        .unwrap();
    for name in ["Organization readiness", "Prior readiness"] {
        assert!(
            chart_page.contains(name),
            "chart lacks color key for {name}: {chart_page}"
        );
    }
}

#[test]
fn pptx_identity_footer_wraps_within_its_measured_box() {
    let mut d = fixture();
    d.document_id = "WW ".repeat(60);
    let r = render(&d, ReportFormat::Pptx);
    let slide = zip_part(&r.bytes, "ppt/slides/slide1.xml");
    let footer = slide
        .split("<p:cNvPr id=\"4\"")
        .nth(1)
        .unwrap()
        .split("</p:sp>")
        .next()
        .unwrap();
    assert!(
        footer.matches("<a:p>").count() >= 2,
        "identity must wrap rather than extend beyond slide width"
    );
}
#[test]
fn pptx_rejects_oversized_identity_footer() {
    let mut d = fixture();
    d.document_id = "W".repeat(256);
    d.revision = "W".repeat(256);
    assert!(render_report(&d, &ReportRenderOptions::new(ReportFormat::Pptx)).is_err());
}

#[test]
fn inspection_rejects_case_insensitive_duplicate_html_attributes() {
    let rendered = render(&fixture(), ReportFormat::Html);
    let original = String::from_utf8(rendered.bytes.clone()).unwrap();
    let modified = original.replace(
        "<meta http-equiv=",
        "<meta HTTP-EQUIV=\"refresh\" CONTENT=\"0;url=https://example.com\" http-equiv=",
    );
    let modified = rebind_bytes(rendered, modified.into_bytes());
    assert!(inspect_report(&modified.bytes, &modified.manifest).is_err());
}
