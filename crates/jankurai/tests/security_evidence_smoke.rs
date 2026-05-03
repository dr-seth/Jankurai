use std::fs;
use std::process::Command;
use tempfile::tempdir;

use jankurai::validation::{self, ArtifactSchema};

fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_jankurai")
}

#[test]
fn security_run_writes_valid_evidence_and_log() {
    let repo = tempdir().unwrap();
    fs::create_dir_all(repo.path().join("tools")).unwrap();
    fs::write(
        repo.path().join("tools/security-lane.sh"),
        "#!/usr/bin/env bash\necho ok\nexit 0\n",
    )
    .unwrap();

    let evidence_path = repo.path().join("out/evidence.json");
    let status = Command::new(binary_path())
        .arg("security")
        .arg("run")
        .arg(repo.path())
        .arg("--script")
        .arg("tools/security-lane.sh")
        .arg("--out")
        .arg(&evidence_path)
        .status()
        .unwrap();
    assert!(status.success(), "security run failed");

    let text = fs::read_to_string(&evidence_path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    validation::validate_value(repo.path(), ArtifactSchema::SecurityEvidence, &value).unwrap();

    assert_eq!(value["exit_code"], 0);
    assert_eq!(value["lane"], "security");
    assert_eq!(value["wrapper"]["strict"], false);

    let log_rel = value["log_path"].as_str().unwrap();
    let log_abs = repo.path().join(log_rel);
    let log_text = fs::read_to_string(&log_abs).unwrap();
    assert!(!log_text.is_empty());
    assert!(log_text.contains("ok"));

    assert!(
        value["commands"][0]["status"] == "ran",
        "{:?}",
        value["commands"]
    );
}

#[test]
fn security_run_records_non_zero_exit_in_evidence() {
    let repo = tempdir().unwrap();
    fs::create_dir_all(repo.path().join("tools")).unwrap();
    fs::write(
        repo.path().join("tools/security-lane.sh"),
        "#!/usr/bin/env bash\necho boom\nexit 7\n",
    )
    .unwrap();

    let evidence_path = repo.path().join("out/evidence.json");
    let status = Command::new(binary_path())
        .arg("security")
        .arg("run")
        .arg(repo.path())
        .arg("--script")
        .arg("tools/security-lane.sh")
        .arg("--out")
        .arg(&evidence_path)
        .status()
        .unwrap();
    assert!(!status.success(), "expected non-zero process exit");

    let text = fs::read_to_string(&evidence_path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    validation::validate_value(repo.path(), ArtifactSchema::SecurityEvidence, &value).unwrap();

    assert_eq!(value["exit_code"], 7);
    assert!(value["commands"][0]["status"] == "failed");
}

#[test]
fn security_run_collects_jankurai_security_step_lines() {
    let repo = tempdir().unwrap();
    fs::create_dir_all(repo.path().join("tools")).unwrap();
    let script = r#"#!/usr/bin/env bash
printf '%s\n' 'jankurai-security-step={"label":"step-a","tool":"t1","shell_command":"true","status":"ran","advisory":false,"exit_code":0}'
printf '%s\n' 'jankurai-security-step={"label":"step-b","shell_command":"true","status":"skipped","advisory":true}'
exit 0
"#;
    fs::write(repo.path().join("tools/security-lane.sh"), script).unwrap();

    let evidence_path = repo.path().join("out/evidence.json");
    let status = Command::new(binary_path())
        .arg("security")
        .arg("run")
        .arg(repo.path())
        .arg("--script")
        .arg("tools/security-lane.sh")
        .arg("--out")
        .arg(&evidence_path)
        .status()
        .unwrap();
    assert!(status.success(), "security run failed");

    let text = fs::read_to_string(&evidence_path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    validation::validate_value(repo.path(), ArtifactSchema::SecurityEvidence, &value).unwrap();

    let cmds = value["commands"].as_array().unwrap();
    assert_eq!(cmds.len(), 2, "{cmds:?}");
    assert_eq!(cmds[0]["label"], "step-a");
    assert_eq!(cmds[0]["status"], "ran");
    assert_eq!(cmds[1]["label"], "step-b");
    assert_eq!(cmds[1]["status"], "skipped");
    assert_eq!(cmds[1]["advisory"], true);
}
