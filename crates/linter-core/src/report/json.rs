use crate::models::report::AuditReport;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsonReportError {
    #[error("Failed to serialize report to JSON: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub fn generate_json_report(report: &AuditReport) -> Result<String, JsonReportError> {
    Ok(serde_json::to_string_pretty(report)?)
}
