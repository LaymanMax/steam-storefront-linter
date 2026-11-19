use linter_core::models::report::AuditReport;
use linter_core::parser::json_import::get_example_listing_json;
use linter_core::rules::profile::AuditProfile;

#[test]
fn test_sample_report_generation() {
    let example_json = get_example_listing_json();
    let listing = linter_core::import_listing_json(example_json).expect("valid json");

    let report = linter_core::run_audit(listing, None, AuditProfile::Balanced, 30);
    assert_eq!(report.listing.name, "Dragon Forge: Chronicles");
    assert_eq!(report.summary.critical, 0);
    assert!(report.summary.passed > 5);
}

#[test]
fn test_invalid_json_import() {
    let invalid = "{ invalid json: true";
    let res = linter_core::import_listing_json(invalid);
    assert!(res.is_err());
}

#[test]
fn test_html_and_json_report_exports() {
    let example_json = get_example_listing_json();
    let listing = linter_core::import_listing_json(example_json).expect("valid json");
    let report = linter_core::run_audit(listing, None, AuditProfile::Balanced, 30);

    let json_str = linter_core::generate_json_report(&report).expect("json export");
    let parsed: AuditReport = serde_json::from_str(&json_str).expect("reparse json report");
    assert_eq!(parsed.schema_version, "1.0.0");
    assert_eq!(parsed.listing.app_id, 2539430);

    let html_str = linter_core::generate_html_report(&report);
    assert!(html_str.contains("Dragon Forge: Chronicles"));
    assert!(html_str.contains("<!DOCTYPE html>"));
}
