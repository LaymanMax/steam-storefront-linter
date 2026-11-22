use crate::models::benchmark::{BenchmarkMethodology, BenchmarkResult, CohortInfo};
use crate::models::finding::{Finding, Severity};
use crate::models::listing::{FieldProvenance, StoreListing};
use crate::rules::profile::AuditProfile;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub const REPORT_SCHEMA_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub critical: usize,
    pub warnings: usize,
    pub info: usize,
    pub passed: usize,
    pub unknown: usize,
    pub total_checks: usize,
    pub section_coverage: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDataQuality {
    pub fields_evaluated: usize,
    pub fields_known: usize,
    pub fields_missing_from_source: usize,
    pub data_completeness_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub schema_version: String,
    pub audit_id: String,
    pub generated_at: DateTime<Utc>,
    pub audit_profile: AuditProfile,
    pub listing: StoreListing,
    pub summary: AuditSummary,
    pub findings: Vec<Finding>,
    pub benchmarks: Vec<BenchmarkResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cohort: Option<CohortInfo>,
    pub methodology: BenchmarkMethodology,
    pub data_quality: AuditDataQuality,
    pub provenance: HashMap<String, FieldProvenance>,
}

impl AuditReport {
    pub fn new(
        listing: StoreListing,
        findings: Vec<Finding>,
        benchmarks: Vec<BenchmarkResult>,
        cohort: Option<CohortInfo>,
        methodology: BenchmarkMethodology,
        audit_profile: AuditProfile,
    ) -> Self {
        let mut critical = 0;
        let mut warnings = 0;
        let mut info = 0;
        let mut passed = 0;
        let mut unknown = 0;

        for f in &findings {
            match f.severity {
                Severity::Critical => critical += 1,
                Severity::Warning => warnings += 1,
                Severity::Info => {
                    info += 1;
                    if f.id.ends_with(".unknown") || f.id.ends_with(".missing_data") {
                        unknown += 1;
                    }
                }
                Severity::Pass => passed += 1,
            }
        }

        let total_checks = findings.len();

        // Calculate coverage by category
        let mut category_totals: HashMap<String, (usize, usize)> = HashMap::new();
        for f in &findings {
            let cat = f.category.display_name().to_string();
            let entry = category_totals.entry(cat).or_insert((0, 0));
            entry.0 += 1; // total
            if f.severity == Severity::Pass {
                entry.1 += 1; // passed
            }
        }

        let mut section_coverage = HashMap::new();
        for (cat, (tot, pass)) in category_totals {
            let pct = if tot > 0 {
                (pass as f64 / tot as f64) * 100.0
            } else {
                0.0
            };
            section_coverage.insert(cat, (pct * 10.0).round() / 10.0);
        }

        // Data quality statistics based on provenance
        let fields_evaluated = listing.provenance.len();
        let fields_known = listing.provenance.values().filter(|p| p.known).count();
        let fields_missing_from_source = fields_evaluated.saturating_sub(fields_known);
        let data_completeness_pct = if fields_evaluated > 0 {
            ((fields_known as f64 / fields_evaluated as f64) * 100.0 * 10.0).round() / 10.0
        } else {
            100.0
        };

        let data_quality = AuditDataQuality {
            fields_evaluated,
            fields_known,
            fields_missing_from_source,
            data_completeness_pct,
        };

        let summary = AuditSummary {
            critical,
            warnings,
            info,
            passed,
            unknown,
            total_checks,
            section_coverage,
        };

        Self {
            schema_version: REPORT_SCHEMA_VERSION.to_string(),
            audit_id: Uuid::new_v4().to_string(),
            generated_at: Utc::now(),
            audit_profile,
            provenance: listing.provenance.clone(),
            listing,
            summary,
            findings,
            benchmarks,
            cohort,
            methodology,
            data_quality,
        }
    }
}
