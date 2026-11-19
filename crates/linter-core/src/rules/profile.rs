use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuditProfile {
    #[default]
    Balanced,
    Strict,
}

impl AuditProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditProfile::Balanced => "balanced",
            AuditProfile::Strict => "strict",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "balanced" => Some(AuditProfile::Balanced),
            "strict" => Some(AuditProfile::Strict),
            _ => None,
        }
    }
}
