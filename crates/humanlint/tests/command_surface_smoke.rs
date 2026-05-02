use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_humanlint"))
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

#[test]
fn new_planner_commands_emit_stable_json_and_markdown() {
    let repo = tempdir().unwrap();
    assert_eq!(fs::read_dir(repo.path()).unwrap().count(), 0);

    let (registry, registry_md) = run_command(&repo.path().to_path_buf(), &["registry"]);
    assert_eq!(registry["command"], "humanlint registry");
    assert_eq!(registry["status"], "complete");
    assert!(registry_md.starts_with("# humanlint Registry"));

    let (cell, cell_md) = run_command(
        &repo.path().to_path_buf(),
        &["cell", "--cell-id", "demo-cell"],
    );
    assert_eq!(cell["command"], "humanlint cell");
    assert_eq!(cell["status"], "complete");
    assert_eq!(cell["mode"], "install-ready");
    assert_eq!(cell["cell_id"], "demo-cell");
    assert_eq!(cell["owner"], "workspace");
    assert!(cell_md.starts_with("# humanlint Cell Plan"));

    let (migrate, migrate_md) = run_command(&repo.path().to_path_buf(), &["migrate"]);
    assert_eq!(migrate["command"], "humanlint migrate");
    assert_eq!(migrate["status"], "complete");
    assert!(migrate_md.starts_with("# humanlint Migration Plan"));

    let (bench, bench_md) = run_command(&repo.path().to_path_buf(), &["bench"]);
    assert_eq!(bench["command"], "humanlint bench");
    assert_eq!(bench["status"], "complete");
    assert!(bench_md.starts_with("# humanlint Bench Plan"));

    let (certify, certify_md) = run_command(&repo.path().to_path_buf(), &["certify"]);
    assert_eq!(certify["command"], "humanlint certify");
    assert_eq!(certify["status"], "complete");
    assert!(certify_md.starts_with("# humanlint Certification Plan"));

    let (govern, govern_md) = run_command(&repo.path().to_path_buf(), &["govern"]);
    assert_eq!(govern["command"], "humanlint govern");
    assert_eq!(govern["status"], "complete");
    assert!(govern_md.starts_with("# humanlint Govern"));

    let plan_path = repo.path().join("repair-plan.json");
    fs::write(
        &plan_path,
        serde_json::json!({
            "schema_version": "1.0.0",
            "source_report": "agent/repo-score.json",
            "generated_at": "0",
            "target_stack_id": "humanlint:v0.4",
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
                "human_review_required": true,
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
    assert_eq!(repair["dry_run"], true);
    assert_eq!(repair["planned_packets"], 1);
    assert!(repair_md.starts_with("# humanlint Repair Run"));
    fs::remove_file(&plan_path).unwrap();

    assert_eq!(fs::read_dir(repo.path()).unwrap().count(), 0);
}
