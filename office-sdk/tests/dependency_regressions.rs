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
