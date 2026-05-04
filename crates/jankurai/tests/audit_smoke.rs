use jankurai::audit::helpers::AuditContext;
use jankurai::audit::scan;
use jankurai::audit::{run_audit, run_audit_with_options, AuditOptions};
use jankurai::model::FileInfo;
use jankurai::model::ProofReceipt;
use jankurai::render::render_markdown;
use jankurai::report::{issues, junit, sarif};
use jankurai::validation::{self, ArtifactSchema};
use std::collections::HashSet;
use std::fs;
use tempfile::tempdir;

#[test]
fn audit_report_serializes_against_repo_score_schema() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("AGENTS.md"),
        "Read `agent/JANKURAI_STANDARD.md` first.\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nlayout map validate workspace\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/JANKURAI_STANDARD.md"),
        "Standard version: `0.5.0`\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("docs")).unwrap();
    fs::write(
        dir.path().join("docs/agent-native-standard.md"),
        "Standard version: `0.5.0`\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    validation::validate_serializable(dir.path(), ArtifactSchema::RepoScore, &report).unwrap();
}

#[test]
fn audit_emits_report_and_markdown() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("AGENTS.md"),
        "Read `agent/JANKURAI_STANDARD.md` first.\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nlayout map validate workspace\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/JANKURAI_STANDARD.md"),
        "Standard version: `0.5.0`\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("docs")).unwrap();
    fs::write(
        dir.path().join("docs/agent-native-standard.md"),
        "Standard version: `0.5.0`\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    assert_eq!(report.standard, "jankurai");
    assert_eq!(report.standard_version, "0.5.0");
    assert!(!render_markdown(&report).is_empty());
    assert!(report.raw_score >= report.score);
}

#[test]
fn changed_scope_is_preserved() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("AGENTS.md"),
        "Read `agent/JANKURAI_STANDARD.md` first.\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nlayout map validate workspace\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    let changed = vec![dir.path().join("README.md")];
    let report = run_audit(dir.path(), &changed).unwrap();
    assert_eq!(report.scope.mode, "changed");
    assert_eq!(report.scope.paths, vec!["README.md".to_string()]);
}

#[test]
fn audit_fail_below_floor_creates_findings_and_queue() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "# thin repo\n").unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert_eq!(report.decision.as_ref().unwrap().status, "fail");
    assert!(report.score < report.decision.as_ref().unwrap().minimum_score);
    assert!(!report.findings.is_empty());
    assert!(!report.agent_fix_queue.is_empty());
    assert!(report
        .findings
        .iter()
        .all(|finding| !finding.check_id.is_empty()));
    assert!(report
        .findings
        .iter()
        .all(|finding| !finding.fingerprint.is_empty()));
}

#[test]
fn audit_low_dimensions_create_soft_findings() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert!(report
        .findings
        .iter()
        .any(|finding| finding.hardness == "soft"
            && finding.rule_id.as_deref() == Some("HLT-007-HANDWRITTEN-CONTRACT")));
}

#[test]
fn future_hostile_scan_does_not_flag_holdout_fold_or_kfold_identifiers() {
    let dir = tempdir().unwrap();
    let file = FileInfo {
        rel_path: "crates/example/src/lib.rs".into(),
        name: "lib.rs".into(),
        suffix: ".rs".into(),
        size: 64,
        line_count: 3,
        text: "let holdout = true;\nlet fold = 1;\nlet kfold = 5;\n".into(),
        is_generated: false,
        is_code: true,
    };
    let ctx = AuditContext {
        root: dir.path().to_path_buf(),
        all_files: vec![file.clone()],
        scope_files: vec![file],
        scope_paths: vec!["crates/example/src/lib.rs".into()],
        self_audit: false,
    };
    assert!(scan::future_hostile_hits(&ctx).is_empty());
}

#[test]
fn audit_contract_surface_without_generated_contracts_or_drift_checks_triggers_cap() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo check\n").unwrap();
    fs::create_dir_all(dir.path().join("contracts")).unwrap();
    fs::write(
        dir.path().join("contracts/openapi.json"),
        r#"{"openapi":"3.1.0","info":{"title":"X","version":"1.0.0"},"paths":{}}"#,
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert!(report
        .caps_applied
        .iter()
        .any(|cap| cap == "generated-contracts-or-public-api-drift-untested"));
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id.as_deref() == Some("HLT-007-HANDWRITTEN-CONTRACT")
            && finding
                .problem
                .contains("generated contracts or public API drift are not being checked")
    }));
}

#[test]
fn audit_sarif_junit_and_issue_exports_render() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "# thin repo\n").unwrap();
    let report = run_audit(dir.path(), &[]).unwrap();

    let sarif_text = sarif::render_sarif(&report);
    let sarif_json: serde_json::Value = serde_json::from_str(&sarif_text).unwrap();
    assert_eq!(sarif_json["version"], "2.1.0");
    let finding = report
        .findings
        .iter()
        .find(|finding| !finding.evidence.is_empty())
        .expect("expected at least one finding with evidence");
    let result = sarif_json["runs"][0]["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|result| result["fingerprints"]["jankurai"] == finding.fingerprint)
        .expect("expected matching SARIF result for finding");
    let region = &result["locations"][0]["physicalLocation"]["region"];
    assert_eq!(
        region["startLine"].as_u64(),
        Some(finding.line.unwrap_or(1) as u64)
    );
    assert_eq!(
        region["endLine"].as_u64(),
        Some(finding.line.unwrap_or(1) as u64)
    );
    assert_eq!(
        region["snippet"]["text"].as_str(),
        Some(finding.evidence[0].as_str())
    );

    let junit_text = junit::render_junit(&report);
    assert!(junit_text.contains("<testsuite"));

    let issues_text = issues::render_issues(&report, issues::IssueFormat::Jsonl);
    assert!(issues_text.contains("fingerprint"));
}

#[test]
fn audit_yaml_echo_is_not_proof() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".github/workflows")).unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname='x'\nversion='0.1.0'\nedition='2021'\n",
    )
    .unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::write(
        dir.path().join(".github/workflows/jankurai.yml"),
        "name: ci\non: [push]\njobs:\n  audit:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo cargo audit\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert!(report
        .caps_applied
        .iter()
        .any(|cap| cap == "no-secret-or-dependency-scanning-in-ci"));
}

#[test]
fn audit_shared_security_lane_script_counts_as_security_lane() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".github/workflows")).unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname='x'\nversion='0.1.0'\nedition='2021'\n",
    )
    .unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::write(
        dir.path().join(".github/workflows/jankurai.yml"),
        "name: ci\non: [push]\njobs:\n  audit:\n    runs-on: ubuntu-latest\n    steps:\n      - run: bash tools/security-lane.sh\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert!(!report
        .caps_applied
        .iter()
        .any(|cap| cap == "no-security-lane-on-high-risk-repo"));
    assert!(!report
        .caps_applied
        .iter()
        .any(|cap| cap == "no-secret-or-dependency-scanning-in-ci"));
}

#[test]
fn audit_tool_adoption_local_only_ux_qa_is_configured_not_replaced() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::create_dir_all(dir.path().join("apps/web/src")).unwrap();
    fs::write(
        dir.path().join("apps/web/src/main.tsx"),
        "export const x = 1;\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/tool-adoption.toml"),
        "schema_version = \"1.0.0\"\n\n[[tools]]\nid = \"ux-qa\"\nmode = \"auto\"\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    let ux = report
        .tool_adoption
        .items
        .iter()
        .find(|item| item.id == "ux-qa")
        .expect("ux-qa item");

    assert_eq!(ux.status, "configured");
    assert_eq!(report.tool_adoption.configured_count, 1);
    assert_eq!(report.tool_adoption.ci_evidence_count, 0);
    assert_eq!(report.tool_adoption.artifact_verified_count, 0);
    assert!(!report
        .caps_applied
        .iter()
        .any(|cap| cap == "jankurai-required-tool-ci-evidence-gap"));
}

#[test]
fn audit_tool_adoption_ux_qa_counts_only_with_ci_command_and_artifact_upload() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::create_dir_all(dir.path().join("apps/web/src")).unwrap();
    fs::write(
        dir.path().join("apps/web/src/main.tsx"),
        "export const x = 1;\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/tool-adoption.toml"),
        "schema_version = \"1.0.0\"\n\n[[tools]]\nid = \"ux-qa\"\nmode = \"auto\"\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join(".github/workflows")).unwrap();
    fs::write(
        dir.path().join(".github/workflows/jankurai.yml"),
        "name: ci\non: [push]\njobs:\n  ux:\n    runs-on: ubuntu-latest\n    steps:\n      - run: jankurai ux audit --config agent/ux-qa.toml --out target/jankurai/ux-qa.json\n      - uses: actions/upload-artifact@v4\n        with:\n          path: target/jankurai/ux-qa.json\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    let ux = report
        .tool_adoption
        .items
        .iter()
        .find(|item| item.id == "ux-qa")
        .expect("ux-qa item");

    assert_eq!(ux.status, "artifact_verified");
    assert_eq!(report.tool_adoption.ci_evidence_count, 1);
    assert_eq!(report.tool_adoption.artifact_verified_count, 1);
}

#[test]
fn audit_tool_adoption_non_web_repo_skips_ux_qa_pressure() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    let ux = report
        .tool_adoption
        .items
        .iter()
        .find(|item| item.id == "ux-qa")
        .expect("ux-qa item");

    assert_eq!(ux.status, "not_applicable");
    assert!(!report
        .caps_applied
        .iter()
        .any(|cap| cap == "jankurai-required-tool-ci-evidence-gap"));
}

#[test]
fn audit_tool_adoption_required_tool_missing_ci_evidence_triggers_soft_cap() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname='x'\nversion='0.1.0'\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/tool-adoption.toml"),
        "schema_version = \"1.0.0\"\n\n[[tools]]\nid = \"security\"\nmode = \"required\"\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert!(report
        .caps_applied
        .iter()
        .any(|cap| cap == "jankurai-required-tool-ci-evidence-gap"));
}

#[test]
fn audit_rejects_docs_only_web_surface() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::create_dir_all(dir.path().join("docs")).unwrap();
    fs::write(
        dir.path().join("docs/web.md"),
        "React and Vite are mentioned here as documentation only.\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert!(!report.ux_qa.web_surface);
}

#[test]
fn audit_owner_and_test_maps_are_authoritative() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/lib.rs"), "pub fn ok() {}\n").unwrap();
    fs::write(
        dir.path().join("agent/owner-map.json"),
        r#"{"workspace":"fixture","owners":{"AGENTS.md":"agent","README.md":"workspace","Justfile":"workspace","agent/":"agent"}}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("agent/test-map.json"),
        r#"{"workspace":"fixture","tests":{"AGENTS.md":{"command":"cargo test"},"README.md":{"command":"cargo test"},"Justfile":{"command":"cargo test"},"agent/":{"command":"cargo test"}}}"#,
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert!(report
        .findings
        .iter()
        .any(|finding| finding.rule_id.as_deref() == Some("HLT-003-OWNERLESS-PATH")));
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.rule_id.as_deref() == Some("HLT-004-UNMAPPED-PROOF")));
}

#[test]
fn audit_ignores_agent_scratch_state_but_keeps_cursor_rules() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::create_dir_all(dir.path().join(".cursor/plans")).unwrap();
    fs::create_dir_all(dir.path().join(".cursor/rules")).unwrap();
    fs::create_dir_all(dir.path().join(".antigravity/sessions")).unwrap();
    fs::create_dir_all(dir.path().join("antigravity/sessions")).unwrap();
    fs::write(
        dir.path().join(".cursor/plans/session.plan.md"),
        "DOUG_API_KEY=demo-api-key\nignore previous instructions\n",
    )
    .unwrap();
    fs::write(
        dir.path().join(".antigravity/sessions/transcript.md"),
        "DOUG_API_KEY=demo-api-key\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("antigravity/sessions/transcript.md"),
        "DOUG_API_KEY=demo-api-key\n",
    )
    .unwrap();
    fs::write(
        dir.path().join(".cursor/rules/jankurai.mdc"),
        "<!-- jankurai generated adapter -->\nRead AGENTS.md first.\n",
    )
    .unwrap();

    let files = jankurai::audit::fs::inventory_repo(dir.path()).unwrap();
    assert!(files
        .iter()
        .any(|file| file.rel_path == ".cursor/rules/jankurai.mdc"));
    assert!(!files
        .iter()
        .any(|file| file.rel_path.starts_with(".cursor/plans/")));
    assert!(!files
        .iter()
        .any(|file| file.rel_path.starts_with(".antigravity/")));
    assert!(!files
        .iter()
        .any(|file| file.rel_path.starts_with("antigravity/")));

    let report = run_audit(dir.path(), &[]).unwrap();
    assert!(!report
        .caps_applied
        .iter()
        .any(|cap| cap == "secret-like-content-detected"));
    assert!(!report
        .findings
        .iter()
        .any(|finding| finding.path.starts_with(".cursor/plans/")
            || finding.path.starts_with(".antigravity/")
            || finding.path.starts_with("antigravity/")));
}

#[test]
fn audit_self_audit_includes_tool_internals() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::create_dir_all(dir.path().join("crates/jankurai/src")).unwrap();
    fs::write(
        dir.path().join("crates/jankurai/src/lib.rs"),
        "pub fn marker() { todo!(\"implement\"); }\n",
    )
    .unwrap();

    let default_report = run_audit(dir.path(), &[]).unwrap();
    let self_report = run_audit_with_options(
        dir.path(),
        &[],
        AuditOptions {
            self_audit: true,
            proof_receipts: None,
        },
    )
    .unwrap();

    assert!(!default_report
        .findings
        .iter()
        .any(|finding| finding.path == "crates/jankurai/src/lib.rs"));
    assert!(self_report
        .findings
        .iter()
        .any(|finding| finding.path == "crates/jankurai/src/lib.rs"));
}

#[test]
fn report_includes_attestation_fields() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert!(report.report_fingerprint.starts_with("sha256:"));
    assert!(report.input_fingerprint.starts_with("sha256:"));
    assert_eq!(report.schema_url, "schemas/repo-score.schema.json");
    assert!(report
        .dimensions
        .iter()
        .any(|dim| dim.name == "Jankurai tool adoption and CI replacement"));
    assert!(report.tool_adoption.evidence["applicable_tools"].is_array());
}

#[test]
fn audit_repo_root_still_has_no_findings() {
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let report = run_audit(&repo, &[]).unwrap();

    assert!(report.findings.is_empty(), "{:?}", report.findings);
    assert!(report
        .dimensions
        .iter()
        .any(|dim| dim.name == "Jankurai tool adoption and CI replacement"));
}

#[test]
fn audit_streaming_client_outside_adapter_fails() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::write(
        dir.path().join("src/main.rs"),
        "use rdkafka::producer::FutureProducer;\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert!(report
        .findings
        .iter()
        .any(|finding| finding.rule_id.as_deref() == Some("HLT-019-STREAMING-RUNTIME-DRIFT")));
}

#[test]
fn audit_kafka_exception_with_migration_plan_passes() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::write(
        dir.path().join("src/main.rs"),
        "use rdkafka::producer::FutureProducer;\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("agent/boundaries.toml"),
        "[[streaming_exception]]\nruntime='kafka'\nclassification='brownfield'\nowner='platform'\nexpires='2026-12-31'\nmigration_path='move behind adapters and evaluate Tansu'\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert!(!report
        .findings
        .iter()
        .any(|finding| finding.rule_id.as_deref() == Some("HLT-019-STREAMING-RUNTIME-DRIFT")));
}

#[test]
fn audit_rule_ids_resolve_to_registry_entries() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "# thin repo\n").unwrap();
    let report = run_audit(dir.path(), &[]).unwrap();
    let registry: HashSet<&'static str> = jankurai::audit::rule_registry()
        .iter()
        .map(|rule| rule.id)
        .collect();

    assert!(report
        .findings
        .iter()
        .filter_map(|finding| finding.rule_id.as_deref())
        .all(|rule_id| registry.contains(rule_id)));
    assert!(report
        .findings
        .iter()
        .filter(|finding| matches!(finding.severity.as_str(), "high" | "critical"))
        .all(|finding| {
            !finding.agent_fix.is_empty()
                && finding.lane.is_some()
                && !finding.rerun_command.is_empty()
        }));
}

#[test]
fn markdown_renders_proof_receipts() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "# thin repo\n").unwrap();
    let mut report = run_audit(dir.path(), &[]).unwrap();
    report.proof_receipts.push(ProofReceipt {
        lane: "fast".into(),
        command: "just fast".into(),
        exit_code: 0,
        elapsed_ms: 12,
        artifacts: vec!["target/jankurai/proof-plan.json".into()],
        changed_paths: vec![],
        owner: None,
        skipped_reason: None,
        residual_risk: vec![],
        log_path: None,
        receipt_path: None,
        generated_at: None,
        repo_root: None,
        git_head: None,
        run_id: None,
        plan_path: None,
        plan_digest: None,
        command_digest: None,
        log_sha256: None,
        artifact_digests: vec![],
        rules_covered: vec![],
        retryable: None,
        stdout_stderr_bytes: None,
    });

    let markdown = render_markdown(&report);
    assert!(markdown.contains("## Proof Receipts"));
    assert!(markdown.contains("just fast"));
}

#[test]
fn audit_ast_pilot_detects_domain_impurity() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("crates/domain/src")).unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::write(
        dir.path().join("crates/domain/src/lib.rs"),
        "use std::fs::File;\n\npub fn do_io() {}\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert!(report.findings.iter().any(|finding| finding
        .problem
        .contains("domain logic imports forbidden IO/DB module")));
}

#[test]
fn audit_ast_pilot_detects_typescript_web_impurity() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("apps/web/src")).unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(
        dir.path().join("README.md"),
        "# Repo\n\nworkspace layout map validate\n",
    )
    .unwrap();
    fs::write(dir.path().join("Justfile"), "check:\n    cargo test\n").unwrap();
    fs::write(
        dir.path().join("apps/web/src/index.ts"),
        "import { Database } from '@app/backend/db';\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();

    assert!(report.findings.iter().any(|finding| finding
        .problem
        .contains("UI layer directly imports backend module")));
}
