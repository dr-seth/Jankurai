use humanlint::audit::run_audit;
use std::fs;
use tempfile::tempdir;

fn thin_repo(dir: &std::path::Path) {
    fs::write(dir.join("README.md"), "# thin repo\n").unwrap();
}

fn minimal_boundaries_toml() -> &'static str {
    r#"
[stack]
id = "fixture-stack"
version = "0.1.0"

[queues]
adapter_paths = ["a/"]
event_contract_paths = []
generated_type_paths = ["g/"]
client_markers = ["m"]
"#
}

#[test]
fn audit_ingests_valid_boundaries_manifest_summary() {
    let dir = tempdir().unwrap();
    thin_repo(dir.path());
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/boundaries.toml"),
        minimal_boundaries_toml(),
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    let art = report.boundaries.artifact.as_ref().expect("artifact");
    assert_eq!(art.path, "agent/boundaries.toml");
    assert!(art.content_fingerprint.starts_with("sha256:"));
    assert_eq!(art.stack_id, "fixture-stack");
    assert_eq!(art.stack_version.as_deref(), Some("0.1.0"));
    assert_eq!(art.adapter_path_count, 1);
    assert_eq!(art.event_contract_path_count, 0);
    assert_eq!(art.generated_type_path_count, 1);
    assert_eq!(art.client_marker_count, 1);
    assert_eq!(art.streaming_exception_count, 0);
}

#[test]
fn audit_invalid_boundaries_toml_leaves_artifact_none() {
    let dir = tempdir().unwrap();
    thin_repo(dir.path());
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/boundaries.toml"),
        "[stack]\nid = \"only\"\n",
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    assert!(report.boundaries.artifact.is_none());
}

#[test]
fn audit_without_boundaries_file_leaves_artifact_none() {
    let dir = tempdir().unwrap();
    thin_repo(dir.path());

    let report = run_audit(dir.path(), &[]).unwrap();
    assert!(report.boundaries.artifact.is_none());
}
