use jankurai::audit::run_audit;
use std::fs;
use tempfile::tempdir;

fn thin_repo(dir: &std::path::Path) {
    fs::write(dir.join("README.md"), "# thin repo\n").unwrap();
}

fn one_report(decision: &str, summary: (u64, u64)) -> serde_json::Value {
    let (errors, warnings) = summary;
    serde_json::json!({
        "schemaVersion": "1.2.0",
        "toolVersion": "0.4.0",
        "url": "about:blank",
        "checkedAt": "2026-05-02T12:00:00.000Z",
        "viewport": { "width": 1280, "height": 720 },
        "metrics": {
            "scrollWidth": 1280,
            "clientWidth": 1280,
            "scrollHeight": 720,
            "clientHeight": 720
        },
        "elements": [],
        "violations": [],
        "artifacts": [],
        "summary": { "errors": errors, "warnings": warnings, "byRule": {} },
        "decision": decision
    })
}

fn evidence_report() -> serde_json::Value {
    let mut report = one_report("block", (1, 0));
    report["schemaVersion"] = serde_json::json!("1.3.0");
    report["artifacts"] = serde_json::json!([
        {
            "kind": "screenshot",
            "path": "target/jankurai/ux-qa/local.png",
            "viewport": { "width": 1280, "height": 720 }
        },
        {
            "kind": "accessibility",
            "path": "target/jankurai/ux-qa/local.a11y.json",
            "viewport": { "width": 1280, "height": 720 }
        }
    ]);
    report["stateCoverage"] = serde_json::json!({
        "required": ["loading", "success"],
        "declared": ["success"],
        "missing": ["loading"]
    });
    report["artifactCoverage"] = serde_json::json!({
        "required": ["screenshot", "aria-snapshot", "accessibility"],
        "present": ["screenshot", "accessibility"],
        "missing": ["aria-snapshot"]
    });
    report["accessibility"] = serde_json::json!({
        "violations": 2,
        "incomplete": 1,
        "passes": 7,
        "artifactPath": "target/jankurai/ux-qa/local.a11y.json"
    });
    report
}

fn ux_qa_envelope(reports: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({ "reports": reports })
}

#[test]
fn audit_ingests_valid_ux_qa_json_artifact_summary() {
    let dir = tempdir().unwrap();
    thin_repo(dir.path());
    fs::create_dir_all(dir.path().join("target/jankurai")).unwrap();
    let env = ux_qa_envelope(vec![one_report("pass", (0, 0))]);
    fs::write(
        dir.path().join("target/jankurai/ux-qa.json"),
        serde_json::to_string_pretty(&env).unwrap(),
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    let art = report.ux_qa.artifact.as_ref().expect("artifact summary");
    assert_eq!(art.path, "target/jankurai/ux-qa.json");
    assert_eq!(art.report_count, 1);
    assert_eq!(art.worst_decision, "pass");
    assert_eq!(art.total_violations, 0);
    assert_eq!(art.summary_errors, 0);
    assert_eq!(art.summary_warnings, 0);
    assert_eq!(art.reports_missing_required_states, 0);
    assert!(art.artifact_counts_by_kind.is_empty());
    assert_eq!(art.accessibility_violation_total, 0);
}

#[test]
fn audit_ux_qa_worst_decision_orders_block_over_pass() {
    let dir = tempdir().unwrap();
    thin_repo(dir.path());
    fs::create_dir_all(dir.path().join("target/jankurai")).unwrap();
    let env = ux_qa_envelope(vec![
        one_report("pass", (0, 0)),
        one_report("block", (0, 0)),
    ]);
    fs::write(
        dir.path().join("target/jankurai/ux-qa.json"),
        serde_json::to_string(&env).unwrap(),
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    let art = report.ux_qa.artifact.as_ref().unwrap();
    assert_eq!(art.report_count, 2);
    assert_eq!(art.worst_decision, "block");
}

#[test]
fn audit_ux_qa_aggregates_summary_counts() {
    let dir = tempdir().unwrap();
    thin_repo(dir.path());
    fs::create_dir_all(dir.path().join("target/jankurai")).unwrap();
    let env = ux_qa_envelope(vec![
        one_report("warn", (2, 5)),
        one_report("review", (1, 0)),
    ]);
    fs::write(
        dir.path().join("target/jankurai/ux-qa.json"),
        serde_json::to_string(&env).unwrap(),
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    let art = report.ux_qa.artifact.as_ref().unwrap();
    assert_eq!(art.summary_errors, 3);
    assert_eq!(art.summary_warnings, 5);
    assert_eq!(art.worst_decision, "review");
}

#[test]
fn audit_ux_qa_aggregates_state_artifact_and_accessibility_evidence() {
    let dir = tempdir().unwrap();
    thin_repo(dir.path());
    fs::create_dir_all(dir.path().join("target/jankurai")).unwrap();
    let env = ux_qa_envelope(vec![evidence_report()]);
    fs::write(
        dir.path().join("target/jankurai/ux-qa.json"),
        serde_json::to_string(&env).unwrap(),
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    let art = report.ux_qa.artifact.as_ref().unwrap();
    assert_eq!(art.reports_missing_required_states, 1);
    assert_eq!(art.missing_state_names, vec!["loading".to_string()]);
    assert_eq!(art.artifact_counts_by_kind.get("screenshot"), Some(&1));
    assert_eq!(art.artifact_counts_by_kind.get("accessibility"), Some(&1));
    assert_eq!(art.reports_missing_required_artifacts, 1);
    assert_eq!(
        art.missing_artifact_kinds,
        vec!["aria-snapshot".to_string()]
    );
    assert_eq!(art.reports_missing_required_accessibility_artifact, 0);
    assert_eq!(art.accessibility_violation_total, 2);
    assert_eq!(art.accessibility_incomplete_total, 1);
    assert_eq!(art.accessibility_pass_total, 7);
}

#[test]
fn audit_adds_findings_for_incomplete_validated_ux_evidence() {
    let dir = tempdir().unwrap();
    thin_repo(dir.path());
    fs::create_dir_all(dir.path().join("target/jankurai")).unwrap();
    let mut report_value = evidence_report();
    report_value["artifactCoverage"]["missing"] =
        serde_json::json!(["aria-snapshot", "accessibility"]);
    let env = ux_qa_envelope(vec![report_value]);
    fs::write(
        dir.path().join("target/jankurai/ux-qa.json"),
        serde_json::to_string(&env).unwrap(),
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.rule_id.as_deref() == Some("HLT-013-RENDERED-UX-GAP")));
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.rule_id.as_deref() == Some("HLT-014-A11Y-GAP")));
}

#[test]
fn audit_invalid_ux_qa_json_leaves_artifact_none() {
    let dir = tempdir().unwrap();
    thin_repo(dir.path());
    fs::create_dir_all(dir.path().join("target/jankurai")).unwrap();
    fs::write(dir.path().join("target/jankurai/ux-qa.json"), "{}\n").unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    assert!(report.ux_qa.artifact.is_none());
}

#[test]
fn audit_without_ux_qa_file_leaves_artifact_none() {
    let dir = tempdir().unwrap();
    thin_repo(dir.path());

    let report = run_audit(dir.path(), &[]).unwrap();
    assert!(report.ux_qa.artifact.is_none());
}
