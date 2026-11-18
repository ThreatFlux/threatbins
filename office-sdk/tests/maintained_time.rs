#![cfg(feature = "report")]

use office_sdk::{OfficeDocument, Timestamp, pptx::PptxDocument, xlsx::XlsxDocument};
use std::io::Cursor;
use zip::ZipArchive;

#[test]
fn spreadsheet_and_presentation_preserve_metadata_instants_and_precision() {
    // An offset input must be normalized without changing the instant, including
    // negative fractional Unix time. XML retains the existing explicit +00:00.
    let created: Timestamp = "1970-01-01T01:29:59.123456789+01:30".parse().unwrap();
    let modified: Timestamp = "2026-11-18T10:20:30.123456-07:00".parse().unwrap();
    let mut spreadsheet = XlsxDocument::new();
    spreadsheet.created = created;
    spreadsheet.modified = modified;
    let mut presentation = PptxDocument::new();
    presentation.created = created;
    presentation.modified = modified;

    for bytes in [
        spreadsheet.to_bytes().unwrap(),
        presentation.to_bytes().unwrap(),
    ] {
        let mut archive = ZipArchive::new(Cursor::new(&bytes)).unwrap();
        let core = office_sdk::utils::read_zip_entry(&mut archive, "docProps/core.xml").unwrap();
        assert!(
            core.contains("1969-12-31T23:59:59.123456789+00:00"),
            "{core}"
        );
        assert!(core.contains("2026-11-18T17:20:30.123456+00:00"), "{core}");
    }
    let spreadsheet = XlsxDocument::from_bytes(&spreadsheet.to_bytes().unwrap()).unwrap();
    assert_eq!(
        (spreadsheet.created, spreadsheet.modified),
        (created, modified)
    );
    let presentation = PptxDocument::from_bytes(&presentation.to_bytes().unwrap()).unwrap();
    assert_eq!(
        (presentation.created, presentation.modified),
        (created, modified)
    );
}

#[cfg(feature = "full")]
#[test]
fn file_metadata_json_preserves_optional_fields_and_fractional_instants() {
    use office_sdk::files::FileMetadata;

    let value = serde_json::json!({
        "title": null, "author": "Example", "subject": null,
        "description": null, "keywords": [], "last_modified_by": null,
        "properties": {},
        "created": "1969-12-31T18:59:59.123456789-05:00",
        "modified": "2026-11-18T12:00:00.123+02:00"
    });
    let metadata: FileMetadata = serde_json::from_value(value).unwrap();
    let encoded = serde_json::to_value(&metadata).unwrap();
    assert_eq!(encoded["created"], "1969-12-31T23:59:59.123456789Z");
    assert_eq!(encoded["modified"], "2026-11-18T10:00:00.123Z");
    assert!(encoded["title"].is_null());
    assert_eq!(encoded["author"], "Example");
    let decoded: FileMetadata = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(decoded.created, metadata.created);
    assert_eq!(decoded.modified, metadata.modified);
    for invalid in [
        "not-a-date",
        "2026-02-30T00:00:00Z",
        "10000-01-01T00:00:00Z",
    ] {
        let mut malformed = encoded.clone();
        malformed["created"] = invalid.into();
        assert!(serde_json::from_value::<FileMetadata>(malformed).is_err());
    }
}

#[cfg(feature = "full")]
#[test]
fn docx_retains_its_existing_whole_second_utc_xml() {
    use office_sdk::docx::DocxDocument;

    let mut document = DocxDocument::new();
    document.metadata.created = "1969-12-31T23:59:59.9Z".parse().unwrap();
    document.metadata.modified = "2026-11-18T10:00:00.123456789Z".parse().unwrap();
    let bytes = document.to_bytes().unwrap();
    let mut archive = ZipArchive::new(Cursor::new(&bytes)).unwrap();
    let core = office_sdk::utils::read_zip_entry(&mut archive, "docProps/core.xml").unwrap();
    assert!(core.contains(">1969-12-31T23:59:59Z</"), "{core}");
    assert!(core.contains(">2026-11-18T10:00:00Z</"), "{core}");
}

#[cfg(feature = "full")]
#[test]
fn mail_date_round_trip_preserves_offset_instant_before_unix_epoch() {
    use office_sdk::eml::EmlDocument;

    let raw = concat!(
        "From: Alice <alice@example.com>\r\n",
        "To: Bob <bob@example.com>\r\n",
        "Date: Wed, 31 Dec 1969 18:59:59 -0500\r\n",
        "Subject: Timestamp compatibility\r\n\r\nHello\r\n"
    );
    let mail = EmlDocument::from_bytes(raw.as_bytes()).unwrap();
    let expected: Timestamp = "1969-12-31T23:59:59Z".parse().unwrap();
    assert_eq!(mail.headers.date, expected);
    assert_eq!(mail.created, expected);
    let bytes = mail.to_bytes().unwrap();
    assert!(String::from_utf8_lossy(&bytes).contains("Date: Wed, 31 Dec 1969 23:59:59 +0000"));
    assert_eq!(
        EmlDocument::from_bytes(&bytes).unwrap().headers.date,
        expected
    );
}
