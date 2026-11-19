use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Warning,
    Info,
    Pass,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Pass => "pass",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    Description,
    Screenshots,
    Trailers,
    Localization,
    Support,
    Features,
    Platforms,
    General,
}

impl FindingCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            FindingCategory::Description => "Description",
            FindingCategory::Screenshots => "Screenshots",
            FindingCategory::Trailers => "Trailers & Video",
            FindingCategory::Localization => "Localization",
            FindingCategory::Support => "Support & Links",
            FindingCategory::Features => "Features & Categories",
            FindingCategory::Platforms => "Platform Support",
            FindingCategory::General => "General Metadata",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FindingEvidence {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub category: FindingCategory,
    pub severity: Severity,
    pub title: String,
    pub message: String,
    pub recommendation: String,
    pub evidence: FindingEvidence,
    pub rule_version: String,
}
