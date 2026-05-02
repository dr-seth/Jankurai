use crate::model::Report;
use serde_json::json;

pub fn render_sarif(report: &Report) -> String {
    let rules = report
        .findings
        .iter()
        .map(|finding| {
            let id = finding
                .rule_id
                .as_deref()
                .unwrap_or("HLT-000-SCORE-DIMENSION");
            json!({
                "id": id,
                "name": finding.check_id,
                "shortDescription": { "text": finding.problem },
                "helpUri": finding.docs_url,
            })
        })
        .collect::<Vec<_>>();
    let results = report
        .findings
        .iter()
        .map(|finding| {
            json!({
                "ruleId": finding.rule_id.as_deref().unwrap_or("HLT-000-SCORE-DIMENSION"),
                "level": sarif_level(&finding.severity),
                "message": { "text": finding.problem },
                "fingerprints": { "jankurai": finding.fingerprint },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": finding.path },
                        "region": { "startLine": finding.line.unwrap_or(1) }
                    }
                }],
                "properties": {
                    "category": finding.category,
                    "hardness": finding.hardness,
                    "confidence": finding.confidence,
                    "evidenceKind": finding.evidence_kind,
                    "rerunCommand": finding.rerun_command,
                    "owner": finding.owner,
                    "lane": finding.lane,
                }
            })
        })
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&json!({
        "version": "2.1.0",
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "jankurai",
                    "version": report.auditor_version,
                    "informationUri": "https://github.com/jeppsontaylor/jankurai",
                    "rules": rules
                }
            },
            "results": results
        }]
    }))
    .unwrap_or_else(|_| "{}".into())
}

fn sarif_level(severity: &str) -> &'static str {
    match severity {
        "critical" | "high" => "error",
        "medium" => "warning",
        _ => "note",
    }
}
