use jankurai::audit::run_audit;
use std::fs;
use tempfile::tempdir;

#[test]
fn audit_flags_incomplete_generated_zone_metadata_missing_source() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "# thin\n").unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/generated-zones.toml"),
        r#"[[zone]]
path = "out/gen.ts"
source = ""
command = "npm run codegen"
"#,
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    let manifest: Vec<_> = report
        .findings
        .iter()
        .filter(|f| {
            f.path == "agent/generated-zones.toml"
                && f.rule_id.as_deref() == Some("HLT-002-GENERATED-MUTATION")
                && f.evidence
                    .iter()
                    .any(|e| e.contains("incomplete reproducibility metadata"))
        })
        .collect();
    assert_eq!(manifest.len(), 1, "{:?}", report.findings);
    assert_eq!(
        manifest[0].rule_id.as_deref(),
        Some("HLT-002-GENERATED-MUTATION")
    );
}

#[test]
fn audit_flags_incomplete_generated_zone_metadata_missing_command() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "# thin\n").unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/generated-zones.toml"),
        r#"[[zone]]
path = "out/gen.ts"
source = "contracts/openapi.yaml"
command = ""
"#,
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    assert!(report.findings.iter().any(|f| {
        f.path == "agent/generated-zones.toml"
            && f.evidence
                .iter()
                .any(|e| e.contains("missing or empty `command`"))
    }));
}

#[test]
fn audit_allows_complete_generated_zone_rows() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "# thin\n").unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/generated-zones.toml"),
        r#"[[zone]]
path = "out/gen.ts"
source = "contracts/openapi.yaml"
command = "npm run codegen"
read_only = true
"#,
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    assert!(!report
        .findings
        .iter()
        .any(|f| f.path == "agent/generated-zones.toml"));
}
