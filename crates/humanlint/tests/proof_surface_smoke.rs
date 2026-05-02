use std::fs;
use std::process::Command;
use tempfile::tempdir;

use humanlint::validation::{self, ArtifactSchema};

fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_humanlint")
}

fn seed_catalog(repo: &std::path::Path) {
    fs::create_dir_all(repo.join("agent")).unwrap();
    fs::write(
        repo.join("agent/owner-map.json"),
        r#"{"owners":{"agent/":"agent","docs/":"standard","tips/":"paper","target/":"workspace"}}"#,
    )
    .unwrap();
    fs::write(
        repo.join("agent/test-map.json"),
        r#"{"tests":{"agent/":{"command":"cargo test -p humanlint","purpose":"agent checks"},"docs/":{"command":"just score","purpose":"audit"}}}"#,
    )
    .unwrap();
    fs::write(
        repo.join("agent/generated-zones.toml"),
        r#"[[zone]]
path = "agent/repo-score.json"
source = "crates/humanlint"
command = "cargo run -p humanlint -- audit . --json agent/repo-score.json --md agent/repo-score.md"
read_only = false
"#,
    )
    .unwrap();
    fs::write(
        repo.join("agent/proof-lanes.toml"),
        r#"[[lane]]
name = "fast"
command = "just fast"
purpose = "fast lane"

[[lane]]
name = "audit"
command = "just score"
purpose = "audit lane"

[[lane]]
name = "security"
command = "just security"
purpose = "security lane"

[[lane]]
name = "release"
command = "just check"
purpose = "release lane"

[[lane]]
name = "fixture"
command = "true"
purpose = "integration test fixture"

[[lane]]
name = "fixture-fail"
command = "false"
purpose = "integration test fixture failure"
"#,
    )
    .unwrap();
}

fn run_lane(repo: &std::path::Path, subcommand: &str, changed: &str) -> serde_json::Value {
    let out = tempdir().unwrap();
    let json_path = out.path().join("plan.json");
    let md_path = out.path().join("plan.md");
    let status = Command::new(binary_path())
        .arg(subcommand)
        .arg(repo)
        .arg("--changed")
        .arg(changed)
        .arg("--out")
        .arg(&json_path)
        .arg("--md")
        .arg(&md_path)
        .status()
        .unwrap();
    assert!(status.success(), "{subcommand} failed");
    serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap()
}

#[test]
fn lane_and_proof_emit_same_plan_for_changed_path() {
    let repo = tempdir().unwrap();
    seed_catalog(repo.path());
    fs::create_dir_all(repo.path().join("docs")).unwrap();
    fs::write(repo.path().join("docs/moonshot.md"), "# moonshot\n").unwrap();

    let lane = run_lane(repo.path(), "lane", "docs/moonshot.md");
    let proof = run_lane(repo.path(), "proof", "docs/moonshot.md");

    assert_eq!(lane["commands"], serde_json::json!(["just score"]));
    assert_eq!(lane["matched_test_map"], serde_json::json!(["docs/"]));
    assert_eq!(lane["required_lanes"], serde_json::json!(["audit"]));
    assert_eq!(lane["commands"], proof["commands"]);
    assert_eq!(lane["required_lanes"], proof["required_lanes"]);
    assert!(lane["planned_runs"][0]["lane"] == "audit");
    assert!(lane["planned_runs"][0]["command"] == "just score");
    validation::validate_value(repo.path(), ArtifactSchema::ProofPlan, &lane).unwrap();
}

#[test]
fn lane_marks_unmapped_paths_as_risky() {
    let repo = tempdir().unwrap();
    seed_catalog(repo.path());

    let plan = run_lane(repo.path(), "lane", "notes/todo.md");
    assert!(plan["risk_notes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|note| note.as_str().unwrap().contains("no test-map proof route")));
    assert!(plan["skipped_lanes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|lane| lane == "full"));
    assert!(plan["required_lanes"].as_array().unwrap().is_empty());
    let entries = plan["skipped_lane_entries"].as_array().unwrap();
    assert!(entries.iter().any(|e| e["lane"] == "full"));
}

#[test]
fn prove_rejects_unsigned_command_without_hatch() {
    let repo = tempdir().unwrap();
    seed_catalog(repo.path());
    let work = repo.path().join("target/humanlint");
    fs::create_dir_all(&work).unwrap();

    let plan_path = work.join("proof-plan.json");
    let plan = serde_json::json!({
        "schema_version": "1.0.0",
        "standard_version": "0.4.0",
        "repo_root": repo.path().display().to_string(),
        "git_head": "unknown",
        "changed_paths": ["docs/moonshot.md"],
        "matched_owner_map": ["docs/"],
        "matched_test_map": ["docs/"],
        "required_lanes": ["audit"],
        "optional_lanes": ["fast", "security", "release"],
        "skipped_lanes": ["fast", "security", "release"],
        "commands": ["rm -f /tmp/nope"],
        "expected_artifacts": ["target/humanlint/proof-receipts/*.json"],
        "risk_notes": [],
        "human_approval_requirements": [],
        "planned_runs": [{
            "lane": "audit",
            "command": "rm -f /tmp/nope",
            "owner": "standard",
            "changed_paths": ["docs/moonshot.md"],
            "artifacts": ["target/humanlint/logs/*.log"],
            "residual_risk": []
        }]
    });
    fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    let output = Command::new(binary_path())
        .arg("prove")
        .arg(repo.path())
        .arg("--plan")
        .arg(&plan_path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("allowlist") || stderr.contains("allow-unsigned"),
        "{stderr}"
    );
}

#[test]
fn prove_writes_receipts_and_logs() {
    let repo = tempdir().unwrap();
    seed_catalog(repo.path());
    let work = repo.path().join("target/humanlint");
    fs::create_dir_all(&work).unwrap();

    let plan_path = work.join("proof-plan.json");
    let receipt_dir = work.join("proof-receipts");
    let evidence_index = work.join("evidence-index.json");
    let plan = serde_json::json!({
        "schema_version": "1.0.0",
        "standard_version": "0.4.0",
        "repo_root": repo.path().display().to_string(),
        "git_head": "unknown",
        "changed_paths": ["docs/moonshot.md"],
        "matched_owner_map": ["docs/"],
        "matched_test_map": ["docs/"],
        "required_lanes": ["audit"],
        "optional_lanes": ["fast", "security", "release"],
        "skipped_lanes": ["fast", "security", "release"],
        "commands": ["true"],
        "expected_artifacts": ["target/humanlint/proof-receipts/*.json"],
        "risk_notes": [],
        "human_approval_requirements": [],
        "planned_runs": [{
            "lane": "fixture",
            "command": "true",
            "owner": "standard",
            "changed_paths": ["docs/moonshot.md"],
            "artifacts": ["target/humanlint/logs/*.log"],
            "residual_risk": []
        }]
    });
    fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    fs::create_dir_all(work.join("security")).unwrap();
    fs::write(work.join("ux-qa.json"), "{\"reports\":[]}\n").unwrap();
    fs::write(work.join("security/evidence.json"), "{}\n").unwrap();
    fs::write(repo.path().join("agent/repo-score.json"), "{\"score\":0}\n").unwrap();
    fs::write(work.join("humanlint.sarif"), "{}\n").unwrap();
    fs::write(work.join("summary.md"), "# summary\n").unwrap();
    fs::write(
        work.join("repair-queue.jsonl"),
        "{\"path\":\"docs/moonshot.md\"}\n",
    )
    .unwrap();

    fs::write(
        repo.path().join("agent/boundaries.toml"),
        r#"
[stack]
id = "proof-fixture"

[queues]
adapter_paths = []
event_contract_paths = []
generated_type_paths = []
"#,
    )
    .unwrap();

    let status = Command::new(binary_path())
        .arg("prove")
        .arg(repo.path())
        .arg("--plan")
        .arg(&plan_path)
        .arg("--out-dir")
        .arg(&receipt_dir)
        .arg("--evidence-index")
        .arg(&evidence_index)
        .status()
        .unwrap();
    assert!(status.success());

    let receipts: Vec<_> = fs::read_dir(&receipt_dir).unwrap().collect();
    assert_eq!(receipts.len(), 1);
    let receipt_path = receipts[0].as_ref().unwrap().path();
    let receipt_value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&receipt_path).unwrap()).unwrap();
    validation::validate_value(repo.path(), ArtifactSchema::ProofReceipt, &receipt_value).unwrap();
    assert_eq!(receipt_value["lane"], "fixture");
    assert_eq!(receipt_value["exit_code"], 0);
    assert!(receipt_value["log_path"].as_str().unwrap().contains("logs"));
    let evidence_raw = fs::read_to_string(&evidence_index).unwrap();
    assert!(evidence_raw.contains("proof-receipts"));
    let evidence_value: serde_json::Value = serde_json::from_str(&evidence_raw).unwrap();
    assert_eq!(evidence_value["schema_version"], "1.2.0");
    assert_eq!(
        evidence_value["ux_qa_report_path"],
        "target/humanlint/ux-qa.json"
    );
    assert_eq!(
        evidence_value["security_evidence_path"],
        "target/humanlint/security/evidence.json"
    );
    assert_eq!(
        evidence_value["repo_score_json_path"],
        "agent/repo-score.json"
    );
    assert_eq!(
        evidence_value["sarif_path"],
        "target/humanlint/humanlint.sarif"
    );
    assert_eq!(
        evidence_value["github_step_summary_path"],
        "target/humanlint/summary.md"
    );
    assert_eq!(
        evidence_value["repair_queue_jsonl_path"],
        "target/humanlint/repair-queue.jsonl"
    );
    assert_eq!(
        evidence_value["boundaries_manifest_path"],
        "agent/boundaries.toml"
    );
    validation::validate_value(repo.path(), ArtifactSchema::EvidenceIndex, &evidence_value)
        .unwrap();
}

#[test]
fn prove_continues_with_failures_when_requested() {
    let repo = tempdir().unwrap();
    seed_catalog(repo.path());
    let work = repo.path().join("target/humanlint");
    fs::create_dir_all(&work).unwrap();

    let plan_path = work.join("proof-plan.json");
    let receipt_dir = work.join("proof-receipts");
    let evidence_index = work.join("evidence-index.json");
    let plan = serde_json::json!({
        "schema_version": "1.0.0",
        "standard_version": "0.4.0",
        "repo_root": repo.path().display().to_string(),
        "git_head": "unknown",
        "changed_paths": ["docs/moonshot.md"],
        "matched_owner_map": ["docs/"],
        "matched_test_map": ["docs/"],
        "required_lanes": ["audit"],
        "optional_lanes": ["fast", "security", "release"],
        "skipped_lanes": ["fast", "security", "release"],
        "commands": ["false", "true"],
        "expected_artifacts": ["target/humanlint/proof-receipts/*.json"],
        "risk_notes": [],
        "human_approval_requirements": [],
        "planned_runs": [
            {
                "lane": "fixture-fail",
                "command": "false",
                "owner": "standard",
                "changed_paths": ["docs/moonshot.md"],
                "artifacts": ["target/humanlint/logs/*.log"],
                "residual_risk": []
            },
            {
                "lane": "fixture",
                "command": "true",
                "owner": "standard",
                "changed_paths": ["docs/moonshot.md"],
                "artifacts": ["target/humanlint/logs/*.log"],
                "residual_risk": []
            }
        ]
    });
    fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    let status = Command::new(binary_path())
        .arg("prove")
        .arg(repo.path())
        .arg("--plan")
        .arg(&plan_path)
        .arg("--out-dir")
        .arg(&receipt_dir)
        .arg("--evidence-index")
        .arg(&evidence_index)
        .arg("--continue-on-error")
        .status()
        .unwrap();
    assert!(!status.success());

    let receipts: Vec<_> = fs::read_dir(&receipt_dir).unwrap().collect();
    assert_eq!(receipts.len(), 2);
    let evidence = fs::read_to_string(&evidence_index).unwrap();
    assert!(evidence.contains("failed_receipts"));
    assert!(evidence.contains("logs"));
}

#[test]
fn prove_shorthand_writes_receipts_with_changed() {
    let repo = tempdir().unwrap();
    seed_catalog(repo.path());
    fs::write(
        repo.path().join("agent/test-map.json"),
        r#"{"tests":{"docs/":{"command":"true","purpose":"audit"}}}"#,
    )
    .unwrap();
    let work = repo.path().join("target/humanlint");
    fs::create_dir_all(&work).unwrap();

    fs::create_dir_all(repo.path().join("docs")).unwrap();
    fs::write(repo.path().join("docs/moonshot.md"), "test").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_humanlint"))
        .arg("prove")
        .arg("--changed")
        .arg("docs/moonshot.md")
        .current_dir(repo.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "prove --changed failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let receipts_dir = repo.path().join("target/humanlint/proof-receipts");
    let receipts: Vec<_> = fs::read_dir(receipts_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    assert_eq!(receipts.len(), 1);
    let receipt_text = fs::read_to_string(&receipts[0]).unwrap();
    assert!(receipt_text.contains("docs/moonshot.md"));
}
