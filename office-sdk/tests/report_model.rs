use office_sdk::report::*;
fn fixture() -> ReportDocument {
    serde_json::from_str(include_str!("fixtures/executive-report.json")).unwrap()
}
#[test]
fn typed_model_hash_is_stable_across_json_object_order() {
    let d = fixture();
    let serialized = serde_json::to_value(&d).unwrap();
    let decoded: ReportDocument = serde_json::from_value(serialized).unwrap();
    assert_eq!(
        report_document_sha256(&d).unwrap(),
        report_document_sha256(&decoded).unwrap()
    );
    assert_eq!(report_document_sha256(&d).unwrap().len(), 64);
}
#[test]
fn model_validation_rejects_mismatched_schema_and_oversized_source() {
    let mut d = fixture();
    d.schema_version = 100;
    assert!(report_document_sha256(&d).is_err());
    let mut d = fixture();
    d.title = "a".repeat(3 * 1024 * 1024);
    assert!(matches!(
        report_document_sha256(&d),
        Err(ReportError::LimitExceeded(_))
    ));
}

#[test]
fn section_first_fixture_retains_its_schema_one_typed_digest() {
    let document: ReportDocument =
        serde_json::from_str(include_str!("fixtures/section-first-report.json")).unwrap();
    assert_eq!(
        report_document_sha256(&document).unwrap(),
        "b1057dbb29f8eb1cb8d5645e9bdd33819a864abd6f8a75b95a8d89bff83ee048"
    );
}
