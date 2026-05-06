use std::fs;
use std::path::PathBuf;
use std::process::Command;

use jankurai::validation::{self, ArtifactSchema};
use tempfile::tempdir;

fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_jankurai"))
}

#[test]
fn badge_command_emits_readme_schema_valid_json() {
    let repo = tempdir().unwrap();
    fs::create_dir_all(repo.path().join("agent")).unwrap();
    fs::write(
        repo.path().join("agent/repo-score.json"),
        serde_json::json!({
            "schema_version": "1.0.0",
            "standard_version": "0.8.0",
            "auditor_version": "0.8.0",
            "score": 95,
            "raw_score": 95,
            "policy": { "minimum_score": 85 },
            "decision": {
                "status": "pass",
                "passed": true,
                "minimum_score": 85,
                "hard_findings": 0,
                "soft_findings": 0
            },
            "findings": [],
            "caps_applied": [],
            "observed_conformance_level": "HL3"
        })
        .to_string(),
    )
    .unwrap();

    let status = Command::new(binary_path())
        .current_dir(repo.path())
        .args(["badge", ".", "--no-readme"])
        .status()
        .unwrap();
    assert!(status.success(), "badge command failed");

    let badge_json = repo.path().join("agent/jankurai-badge.json");
    let badge: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&badge_json).unwrap()).unwrap();
    validation::validate_value(repo.path(), ArtifactSchema::ReadmeBadge, &badge).unwrap();
    assert_eq!(badge["standard"], "jankurai");
    assert_eq!(badge["score"], 95);

    let badge_svg = fs::read_to_string(repo.path().join("agent/jankurai-badge.svg")).unwrap();
    assert!(badge_svg.contains(">95/100<"));
    assert!(!badge_svg.contains("95/100 pass"));
}
