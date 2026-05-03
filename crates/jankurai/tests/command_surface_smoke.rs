use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

use jankurai::commands::bench;
use jankurai::validation::{self, ArtifactSchema};

fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_jankurai"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn run_command(repo: &PathBuf, args: &[&str]) -> (serde_json::Value, String) {
    let out_dir = tempdir().unwrap();
    let json_path = out_dir.path().join("out.json");
    let md_path = out_dir.path().join("out.md");
    let subcommand = args[0];
    let mut cmd = Command::new(binary_path());
    cmd.arg(subcommand)
        .arg(repo)
        .args(&args[1..])
        .arg("--out")
        .arg(&json_path)
        .arg("--md")
        .arg(&md_path);
    let status = cmd.status().unwrap();
    assert!(status.success(), "command failed: {:?}", cmd);
    let json_text = fs::read_to_string(&json_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_text).unwrap();
    let md_text = fs::read_to_string(&md_path).unwrap();
    (json, md_text)
}

fn run_repair_with_draft(
    repo: &PathBuf,
    plan_path: &PathBuf,
    draft_path: &PathBuf,
) -> (serde_json::Value, serde_json::Value) {
    let run_dir = tempdir().unwrap();
    let run_json = run_dir.path().join("repair-run.json");
    let run_md = run_json.with_extension("md");
    let draft_md = draft_path.with_extension("md");
    let status = Command::new(binary_path())
        .arg("repair")
        .arg(repo)
        .arg("--plan")
        .arg(plan_path)
        .arg("--dry-run")
        .arg("--auto-pr")
        .arg("--max-risk")
        .arg("medium")
        .arg("--out")
        .arg(&run_json)
        .arg("--md")
        .arg(&run_md)
        .arg("--pr-draft-out")
        .arg(draft_path)
        .arg("--pr-draft-md")
        .arg(&draft_md)
        .status()
        .unwrap();
    assert!(status.success());
    let run: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&run_json).unwrap()).unwrap();
    let draft: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(draft_path).unwrap()).unwrap();
    (run, draft)
}

#[test]
fn new_planner_commands_emit_stable_json_and_markdown() {
    let repo = tempdir().unwrap();
    assert_eq!(fs::read_dir(repo.path()).unwrap().count(), 0);

    let (registry, registry_md) = run_command(&repo.path().to_path_buf(), &["registry"]);
    assert_eq!(registry["command"], "jankurai registry");
    assert_eq!(registry["status"], "complete");
    assert!(registry_md.starts_with("# jankurai Registry"));
    validation::validate_value(repo.path(), ArtifactSchema::CellRegistry, &registry).unwrap();

    let (cell, cell_md) = run_command(
        &repo.path().to_path_buf(),
        &["cell", "--cell-id", "demo-cell"],
    );
    assert_eq!(cell["command"], "jankurai cell");
    assert_eq!(cell["status"], "complete");
    assert_eq!(cell["mode"], "install-ready");
    assert_eq!(cell["cell_id"], "demo-cell");
    assert_eq!(cell["owner"], "workspace");
    assert!(cell_md.starts_with("# jankurai Cell Plan"));
    validation::validate_value(repo.path(), ArtifactSchema::CellManifest, &cell["manifest"])
        .unwrap();

    let (migrate, migrate_md) = run_command(&repo.path().to_path_buf(), &["migrate"]);
    assert_eq!(migrate["command"], "jankurai migrate");
    assert_eq!(migrate["status"], "complete");
    assert!(migrate_md.starts_with("# jankurai Migration Plan"));

    let (bench, bench_md) = run_command(&repo.path().to_path_buf(), &["bench"]);
    let suite = bench::build_benchmark_suite(repo.path()).unwrap();
    validation::validate_serializable(repo.path(), ArtifactSchema::BenchmarkSuite, &suite).unwrap();
    assert_eq!(bench["suite_id"], "smoke");
    assert!(bench["results"].as_array().unwrap().len() >= 2);
    assert!(bench["summary"]["passed"].as_i64().unwrap() >= 1);
    assert!(bench_md.starts_with("# jankurai Benchmark Report"));
    validation::validate_value(repo.path(), ArtifactSchema::BenchmarkReport, &bench).unwrap();

    let (certify, certify_md) = run_command(&repo.path().to_path_buf(), &["certify"]);
    assert_eq!(certify["standard_version"], "0.4.0");
    assert_eq!(certify["score"], 0);
    assert_eq!(certify["conformance_level"], "HL0");
    assert!(certify_md.starts_with("# jankurai Certification"));
    validation::validate_value(repo.path(), ArtifactSchema::Certification, &certify).unwrap();

    let (govern, govern_md) = run_command(&repo.path().to_path_buf(), &["govern"]);
    assert_eq!(govern["minimum_score"], 85);
    assert_eq!(govern["update_channel"], "stable");
    assert!(govern_md.starts_with("# jankurai Governance Policy"));
    validation::validate_value(repo.path(), ArtifactSchema::GovernancePolicy, &govern).unwrap();

    let plan_path = repo.path().join("repair-plan.json");
    fs::write(
        &plan_path,
        serde_json::json!({
            "schema_version": "1.0.0",
            "source_report": "agent/repo-score.json",
            "generated_at": "0",
            "target_stack_id": "jankurai:v0.4",
            "plan_mode": "dry-run",
            "planned_edits": [{
                "path": "docs/testing.md",
                "operation": "modify",
                "reason": "add docs",
                "finding_fingerprint": "sha256:test",
                "rule_id": "HLT-017-OPAQUE-OBSERVABILITY",
                "apply_strategy": "none",
                "risk_level": "medium",
                "repair_eligibility": "agent-assisted"
            }],
            "planned_commands": ["just fast"],
            "proof_lanes": ["audit"],
            "rollback_guidance": ["restore docs"],
            "human_approval_requirements": [],
            "packets": [{
                "finding_fingerprint": "sha256:test",
                "finding_path": "docs/testing.md",
                "rule_id": "HLT-017-OPAQUE-OBSERVABILITY",
                "check_id": "HLT-017-OPAQUE-OBSERVABILITY",
                "severity": "medium",
                "owner": "standard",
                "lane": "audit",
                "problem": "opaque observability",
                "why": "opaque observability",
                "permission_profile": "docs-only",
                "allowed_paths": ["docs/"],
                "forbidden_paths": ["reference/"],
                "expected_patch_shape": "add docs",
                "required_proof": ["just fast"],
                "stop_conditions": ["stop"],
                "repair_eligibility": "agent-assisted",
                "risk_level": "medium",
                "eligibility_reason": "observability repairs are typically scoped to telemetry and error receipts",
                "human_review_required": false,
                "rollback_guidance": "restore docs"
            }]
        })
        .to_string(),
    )
    .unwrap();
    let (repair, repair_md) = run_command(
        &repo.path().to_path_buf(),
        &["repair", "--plan", plan_path.to_str().unwrap(), "--dry-run"],
    );
    assert_eq!(repair["status"], "complete");
    assert_eq!(repair["execution_mode"], "dry-run");
    assert_eq!(repair["dry_run"], true);
    assert_eq!(repair["auto_pr_status"], "not-requested");
    assert_eq!(repair["planned_packets"], 1);
    assert!(repair["applied_edits"].as_array().unwrap().is_empty());
    assert!(repair["skipped_edits"].as_array().unwrap().is_empty());
    assert!(repair["files_written"].as_array().unwrap().is_empty());
    assert!(repair["proof_evidence_index"].is_null());
    validation::validate_value(repo.path(), ArtifactSchema::RepairRun, &repair).unwrap();
    assert!(repair_md.starts_with("# jankurai Repair Run"));
    fs::remove_file(&plan_path).unwrap();

    assert_eq!(fs::read_dir(repo.path()).unwrap().count(), 0);
}

#[test]
fn repair_command_emits_auto_pr_draft_artifact() {
    let repo = tempdir().unwrap();
    assert_eq!(fs::read_dir(repo.path()).unwrap().count(), 0);
    fs::create_dir_all(repo.path().join("agent")).unwrap();
    fs::write(
        repo.path().join("agent/owner-map.json"),
        r#"{"workspace":"fixture","owners":{"agent/":"agent","docs/":"standard","paper/":"paper","reference/":"read-only","target/":"workspace","crates/":"tools"}}"#,
    )
    .unwrap();
    fs::write(
        repo.path().join("agent/test-map.json"),
        r#"{"workspace":"fixture","tests":{"docs/":{"command":"true","purpose":"fixture docs proof"},"agent/":{"command":"true","purpose":"fixture agent proof"}}}"#,
    )
    .unwrap();
    fs::write(
        repo.path().join("agent/proof-lanes.toml"),
        r#"[[lane]]
name = "audit"
command = "true"
purpose = "fixture proof"
"#,
    )
    .unwrap();
    fs::write(
        repo.path().join("agent/generated-zones.toml"),
        r#"[[zone]]
path = "agent/repo-score.json"
source = "crates/jankurai"
command = "cargo run -p jankurai -- . --json agent/repo-score.json --md agent/repo-score.md"
read_only = false
"#,
    )
    .unwrap();

    let plan_path = repo.path().join("repair-plan.json");
    fs::write(
        &plan_path,
        serde_json::json!({
            "schema_version": "1.0.0",
            "source_report": "agent/repo-score.json",
            "generated_at": "0",
            "target_stack_id": "jankurai:v0.4",
            "plan_mode": "dry-run",
            "planned_edits": [{
                "path": "docs/testing.md",
                "operation": "modify",
                "reason": "add docs",
                "finding_fingerprint": "sha256:test",
                "rule_id": "HLT-017-OPAQUE-OBSERVABILITY",
                "apply_strategy": "append-text",
                "risk_level": "medium",
                "repair_eligibility": "agent-assisted"
            }],
            "planned_commands": ["false"],
            "proof_lanes": ["audit"],
            "rollback_guidance": ["restore docs"],
            "human_approval_requirements": [],
            "packets": [{
                "finding_fingerprint": "sha256:test",
                "finding_path": "docs/testing.md",
                "rule_id": "HLT-017-OPAQUE-OBSERVABILITY",
                "check_id": "HLT-017-OPAQUE-OBSERVABILITY",
                "severity": "medium",
                "owner": "standard",
                "lane": "audit",
                "problem": "opaque observability",
                "why": "opaque observability",
                "permission_profile": "docs-only",
                "allowed_paths": ["docs/"],
                "forbidden_paths": ["reference/"],
                "expected_patch_shape": "add docs",
                "required_proof": ["true"],
                "stop_conditions": ["stop"],
                "repair_eligibility": "agent-assisted",
                "risk_level": "medium",
                "eligibility_reason": "observability repairs are typically scoped to telemetry and error receipts",
                "human_review_required": false,
                "rollback_guidance": "restore docs"
            }]
        })
        .to_string(),
    )
    .unwrap();
    let draft_path = repo.path().join("repair-pr-draft.json");
    let (repair, draft) =
        run_repair_with_draft(&repo.path().to_path_buf(), &plan_path, &draft_path);

    assert_eq!(repair["auto_pr_status"], "eligible-dry-run-only");
    assert_eq!(draft["status"], "draft-only");
    assert!(draft["artifact_links"]
        .as_array()
        .unwrap()
        .iter()
        .any(|link| link == "agent/repo-score.json"));
    assert!(draft["pr_body"]
        .as_str()
        .unwrap()
        .contains("Eligible Packets"));
    validation::validate_value(repo.path(), ArtifactSchema::RepairRun, &repair).unwrap();
    validation::validate_value(repo.path(), ArtifactSchema::RepairPrDraft, &draft).unwrap();
}

#[test]
fn certified_cells_are_schema_valid_and_evidence_bound() {
    let repo = repo_root();

    let (registry, _registry_md) = run_command(&repo, &["registry"]);
    validation::validate_value(&repo, ArtifactSchema::CellRegistry, &registry).unwrap();
    let cells = registry["cells"].as_array().unwrap();
    let audit_log = cells
        .iter()
        .find(|cell| cell["cell_id"] == "audit-log")
        .expect("audit-log cell");
    let crud = cells
        .iter()
        .find(|cell| cell["cell_id"] == "crud-resource")
        .expect("crud-resource cell");
    let rbac = cells
        .iter()
        .find(|cell| cell["cell_id"] == "rbac")
        .expect("rbac cell");
    assert_eq!(audit_log["certification_status"], "certified");
    assert!(audit_log["proof_lanes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|lane| lane == "audit"));
    assert_eq!(crud["dependencies"].as_array().unwrap()[0], "audit-log");
    assert_eq!(rbac["certification_status"], "certified");
    assert_eq!(rbac["dependencies"].as_array().unwrap()[0], "crud-resource");
    assert!(rbac["proof_lanes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|lane| lane == "security"));

    let (cell, _cell_md) = run_command(&repo, &["cell", "--cell-id", "audit-log"]);
    validation::validate_value(&repo, ArtifactSchema::CellManifest, &cell["manifest"]).unwrap();
    assert!(cell["manifest"].is_object());
    assert_eq!(cell["install_plan"]["dry_run"], true);
    assert_eq!(cell["install_plan"]["conflict_policy"], "never-overwrite");

    let (prove, _prove_md) = run_command(
        &repo,
        &["cell", "--cell-id", "audit-log", "--mode", "prove"],
    );
    validation::validate_value(&repo, ArtifactSchema::CellManifest, &prove["manifest"]).unwrap();
    assert!(!prove["certification_evidence"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(!prove["proof_commands"].as_array().unwrap().is_empty());

    let (rbac_prove, _rbac_md) =
        run_command(&repo, &["cell", "--cell-id", "rbac", "--mode", "prove"]);
    validation::validate_value(&repo, ArtifactSchema::CellManifest, &rbac_prove["manifest"])
        .unwrap();
    assert_eq!(rbac_prove["manifest"]["cell_id"], "rbac");
    assert_eq!(rbac_prove["manifest"]["certification_status"], "certified");
}
