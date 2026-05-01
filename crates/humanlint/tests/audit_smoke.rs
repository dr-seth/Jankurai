use humanlint::audit::run_audit;
use humanlint::render::render_markdown;
use std::fs;
use tempfile::tempdir;

#[test]
fn audit_emits_report_and_markdown() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("AGENTS.md"),
        "Read `agent/HUMANLINT_STANDARD.md` first.\n",
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
        dir.path().join("agent/HUMANLINT_STANDARD.md"),
        "Standard version: `0.2.0`\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("docs")).unwrap();
    fs::write(
        dir.path().join("docs/agent-native-standard.md"),
        "Standard version: `0.2.0`\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    assert_eq!(report.standard, "humanlint");
    assert_eq!(report.standard_version, "0.2.0");
    assert!(!render_markdown(&report).is_empty());
    assert!(report.raw_score >= report.score);
}

#[test]
fn changed_scope_is_preserved() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("AGENTS.md"),
        "Read `agent/HUMANLINT_STANDARD.md` first.\n",
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
