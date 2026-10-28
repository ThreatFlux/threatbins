use office_sdk::utils::validate_office_format;
use office_sdk::{OfficeDocument, docx::DocxDocument, pptx::PptxDocument, xlsx::XlsxDocument};
use std::io::{Cursor, Write};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

#[test]
fn office_validation_rejects_unsupported_entry_compression_without_panicking() {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file("[Content_Types].xml", SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"<Types/>").unwrap();
    let mut bytes = writer.finish().unwrap().into_inner();
    let central = bytes
        .windows(4)
        .position(|bytes| bytes == b"PK\x01\x02")
        .expect("central directory");
    // The central entry remains structurally valid, but its compression method
    // cannot be decoded. Format validation must return an error to the caller.
    bytes[central + 10..central + 12].copy_from_slice(&u16::MAX.to_le_bytes());
    let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("valid directory structure");
    assert!(validate_office_format(&mut archive, &["[Content_Types].xml"]).is_err());
}

#[test]
fn office_xml_round_trips_unicode_whitespace_and_references() {
    let text = "Résumé  & <タグ> — λ";
    let mut docx = DocxDocument::new();
    docx.add_paragraph(text).unwrap();
    let parsed = DocxDocument::from_bytes(&docx.to_bytes().unwrap()).unwrap();
    assert!(parsed.get_text().unwrap().contains(text));
    let mut xlsx = XlsxDocument::new();
    xlsx.add_worksheet("Résumé", &[vec![text.to_owned()]])
        .unwrap();
    let parsed = XlsxDocument::from_bytes(&xlsx.to_bytes().unwrap()).unwrap();
    assert_eq!(parsed.get_worksheet_data("Résumé").unwrap()[0][0], text);
    let mut pptx = PptxDocument::new();
    pptx.add_slide("Résumé", &[text]).unwrap();
    let parsed = PptxDocument::from_bytes(&pptx.to_bytes().unwrap()).unwrap();
    assert!(parsed.get_slide_text().unwrap().contains(text.trim()));
}

#[test]
fn xlsx_metadata_and_binary_vba_survive_archive_round_trip() {
    let mut document = XlsxDocument::new();
    document.set_author("Research & Development");
    document.set_title("Quarterly <Report>");
    document.set_subject("Sales & forecasts");
    document.set_description("Unicode: café, 日本語");
    document.set_keywords("Q4, reports");
    document.set_category("Finance");
    let project = vec![0, 0xff, 0x80, 1, 2, 3];
    document.add_vba_macro(project.clone()).unwrap();

    let bytes = document.to_bytes().unwrap();
    let mut archive = ZipArchive::new(Cursor::new(&bytes)).unwrap();
    let types = office_sdk::utils::read_zip_entry(&mut archive, "[Content_Types].xml").unwrap();
    assert!(types.contains("application/vnd.ms-excel.sheet.macroEnabled.main+xml"));
    assert!(types.contains("/xl/vbaProject.bin"));
    let relationships =
        office_sdk::utils::read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels").unwrap();
    assert!(relationships.contains("relationships/vbaProject"));
    assert!(relationships.contains("Target=\"vbaProject.bin\""));
    assert_eq!(
        office_sdk::utils::read_zip_entry_bytes(&mut archive, "xl/vbaProject.bin").unwrap(),
        project
    );

    let parsed = XlsxDocument::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.author, document.author);
    assert_eq!(parsed.title, document.title);
    assert_eq!(parsed.subject, document.subject);
    assert_eq!(parsed.description, document.description);
    assert_eq!(parsed.keywords, document.keywords);
    assert_eq!(parsed.category, document.category);
    assert_eq!(parsed.get_vba_project(), Some(&project));

    let plain = XlsxDocument::new().to_bytes().unwrap();
    let mut archive = ZipArchive::new(Cursor::new(&plain)).unwrap();
    assert!(archive.by_name("xl/vbaProject.bin").is_err());
    assert!(!XlsxDocument::from_bytes(&plain).unwrap().has_vba_macros());
}

#[test]
fn pdf_metadata_decodes_utf16_text_strings() {
    let mut document = office_sdk::pdf::PdfDocument::new();
    document.set_title("Résumé 日本語");
    document.set_author("Research & Développement");
    document.add_a4_page().unwrap();
    let parsed = office_sdk::pdf::PdfDocument::from_bytes(&document.to_bytes().unwrap()).unwrap();
    assert_eq!(parsed.title, document.title);
    assert_eq!(parsed.author, document.author);
}

#[test]
fn pptx_retains_media_notes_and_sdk_animation_definitions_after_two_saves() {
    let mut document = PptxDocument::new();
    document.set_author("R&D <Team>");
    document.set_title("Quarterly & Annual");
    let slide = document.add_slide("Overview", &["Content"]).unwrap();
    let notes = "Speaker notes & <references> 日本語";
    document.add_slide_notes(&slide, notes).unwrap();
    let mut image_bytes = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(image::RgbImage::new(2, 3))
        .write_to(&mut image_bytes, image::ImageFormat::Png)
        .unwrap();
    let image = image_bytes.into_inner();
    document
        .add_image_from_bytes(&image, "image/png", Some("Image"))
        .unwrap();
    let animation = document
        .add_animation(
            "title",
            office_sdk::pptx::AnimationType::Custom("fade & <zoom>".to_string()),
            1.25,
            0.5,
        )
        .unwrap();
    let project = vec![0, 0xff, 0x80, 5];
    document.add_vba_macro(project.clone()).unwrap();

    for _ in 0..2 {
        let bytes = document.to_bytes().unwrap();
        let mut archive = ZipArchive::new(Cursor::new(&bytes)).unwrap();
        assert_eq!(
            office_sdk::utils::read_zip_entry_bytes(&mut archive, "ppt/media/image1.png").unwrap(),
            image
        );
        let presentation =
            office_sdk::utils::read_zip_entry(&mut archive, "ppt/presentation.xml").unwrap();
        assert_eq!(presentation.matches("<sdk:animations ").count(), 1);
        let types = office_sdk::utils::read_zip_entry(&mut archive, "[Content_Types].xml").unwrap();
        assert!(types.contains("application/vnd.ms-powerpoint.presentation.macroEnabled.main+xml"));
        assert!(types.contains("notesSlide1.xml"));
        document = PptxDocument::from_bytes(&bytes).unwrap();
        assert_eq!(document.author, "R&D <Team>");
        assert_eq!(document.title, "Quarterly & Annual");
        assert_eq!(
            document.get_slide_notes(&slide).map(String::as_str),
            Some(notes)
        );
        assert_eq!(document.images["image1"].data, image);
        assert_eq!(
            (
                document.images["image1"].width,
                document.images["image1"].height
            ),
            (2, 3)
        );
        assert_eq!(document.animations[&animation].target_shape, "title");
        assert_eq!(document.animations[&animation].duration, 1.25);
        assert_eq!(document.animations[&animation].delay, 0.5);
        assert!(
            matches!(&document.animations[&animation].animation_type, office_sdk::pptx::AnimationType::Custom(value) if value == "fade & <zoom>")
        );
        assert_eq!(document.get_vba_project(), Some(&project));
    }
}

#[test]
fn pptx_does_not_treat_media_directories_or_video_as_images() {
    let mut document = PptxDocument::new();
    document.add_slide("Overview", &["Content"]).unwrap();
    let bytes = document.to_bytes().unwrap();
    let mut source = ZipArchive::new(Cursor::new(&bytes)).unwrap();
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        let name = entry.name().unwrap().into_owned();
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        std::io::copy(&mut entry, &mut writer).unwrap();
    }
    writer
        .add_directory("ppt/media/", SimpleFileOptions::default())
        .unwrap();
    writer
        .start_file("ppt/media/video.mp4", SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"non-image media").unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    let parsed = PptxDocument::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.slides.len(), 1);
    assert!(parsed.images.is_empty());
}
