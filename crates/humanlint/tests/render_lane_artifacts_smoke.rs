use humanlint::audit::run_audit;
use humanlint::render::render_markdown;
use humanlint::report::github;
use std::fs;
use tempfile::tempdir;

fn thin_repo(dir: &std::path::Path) {
    fs::write(dir.join("README.md"), "# thin repo\n").unwrap();
}

fn minimal_ux_envelope() -> serde_json::Value {
    serde_json::json!({
        "reports": [{
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
            "summary": { "errors": 0, "warnings": 0, "byRule": {} },
            "decision": "warn"
        }]
    })
}

fn minimal_security_envelope() -> serde_json::Value {
    serde_json::json!({
        "schema_version": "1.0.0",
        "standard_version": "0.4.0",
        "generated_at": "2026-05-02T12:00:00.000Z",
        "repo_root": "/tmp/x",
        "lane": "security",
        "wrapper": { "kind": "bash_script", "path": "tools/security-lane.sh", "strict": false },
        "exit_code": 0,
        "elapsed_ms": 9,
        "log_path": "target/humanlint/security/run.log",
        "commands": [{
            "label": "lane",
            "shell_command": "bash tools/security-lane.sh",
            "status": "ran",
            "advisory": false
        }]
    })
}

#[test]
fn markdown_and_github_summary_include_lane_artifacts() {
    let dir = tempdir().unwrap();
    thin_repo(dir.path());
    fs::create_dir_all(dir.path().join("target/humanlint")).unwrap();
    fs::create_dir_all(dir.path().join("target/humanlint/security")).unwrap();

    fs::write(
        dir.path().join("target/humanlint/ux-qa.json"),
        serde_json::to_string(&minimal_ux_envelope()).unwrap(),
    )
    .unwrap();
    fs::write(
        dir.path().join("target/humanlint/security/evidence.json"),
        serde_json::to_string(&minimal_security_envelope()).unwrap(),
    )
    .unwrap();

    let report = run_audit(dir.path(), &[]).unwrap();
    assert!(report.ux_qa.artifact.is_some());
    assert!(report.security_evidence.artifact.is_some());

    let md = render_markdown(&report);
    assert!(md.contains("### Ingested UX QA report (`target/humanlint/ux-qa.json`)"));
    assert!(md.contains("- Worst decision: `warn`"));
    assert!(md.contains("## Security evidence (ingested)"));
    assert!(md.contains("- Source: `target/humanlint/security/evidence.json`"));

    let gh = github::render_step_summary(&report);
    assert!(gh.contains("#### lane artifacts"));
    assert!(gh.contains("ux-qa `target/humanlint/ux-qa.json`"));
    assert!(gh.contains("worst=warn"));
    assert!(gh.contains("security `target/humanlint/security/evidence.json`"));
}
