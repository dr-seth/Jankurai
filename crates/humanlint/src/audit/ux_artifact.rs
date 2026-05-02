//! Load and summarize `target/humanlint/ux-qa.json` for audit reports.

use crate::model::UxQaReportArtifactSummary;
use crate::validation::{self, ArtifactSchema};
use serde_json::Value;
use std::path::Path;

const UX_QA_REPORT_REL: &str = "target/humanlint/ux-qa.json";

pub fn load_report_summary(root: &Path) -> Option<UxQaReportArtifactSummary> {
    let path = root.join(UX_QA_REPORT_REL);
    if !path.is_file() {
        return None;
    }
    let text = std::fs::read_to_string(&path).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    validation::validate_value(root, ArtifactSchema::UxQaReport, &value).ok()?;
    summarize(&value)
}

/// Validate and summarize already-parsed JSON (for tests).
pub fn summarize_validated_report(root: &Path, value: &Value) -> Option<UxQaReportArtifactSummary> {
    validation::validate_value(root, ArtifactSchema::UxQaReport, value).ok()?;
    summarize(value)
}

fn summarize(value: &Value) -> Option<UxQaReportArtifactSummary> {
    let reports = value.get("reports")?.as_array()?;
    let report_count = reports.len();
    let mut total_violations = 0usize;
    let mut summary_errors = 0u64;
    let mut summary_warnings = 0u64;
    let mut worst_rank = 0u8;
    let mut worst_decision = "pass".to_string();

    for report in reports {
        if let Some(arr) = report.get("violations").and_then(Value::as_array) {
            total_violations += arr.len();
        }
        if let Some(summary) = report.get("summary") {
            summary_errors += summary.get("errors").and_then(Value::as_u64).unwrap_or(0);
            summary_warnings += summary.get("warnings").and_then(Value::as_u64).unwrap_or(0);
        }
        let d = report
            .get("decision")
            .and_then(Value::as_str)
            .unwrap_or("pass");
        let rank = decision_rank(d);
        if rank > worst_rank {
            worst_rank = rank;
            worst_decision = d.to_string();
        }
    }

    Some(UxQaReportArtifactSummary {
        path: UX_QA_REPORT_REL.into(),
        report_count,
        worst_decision,
        total_violations,
        summary_errors,
        summary_warnings,
    })
}

fn decision_rank(decision: &str) -> u8 {
    match decision {
        "block" => 4,
        "review" => 3,
        "warn" => 2,
        "pass" => 1,
        _ => 1,
    }
}
