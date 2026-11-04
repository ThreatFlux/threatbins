use office_sdk::report::*;
fn fixture() -> ReportDocument {
    serde_json::from_str(include_str!("fixtures/qbr-report.json")).unwrap()
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
