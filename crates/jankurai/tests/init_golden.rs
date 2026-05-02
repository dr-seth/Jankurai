use jankurai::commands::init;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_jankurai")
}

#[test]
fn init_unknown_profile_errors() {
    let dir = tempdir().unwrap();
    let err = init::run(init::InitArgs {
        repo: dir.path().to_path_buf(),
        apply: false,
        dry_run: true,
        yes: false,
        profile: "not-a-real-profile".into(),
        ide: "all".into(),
        mode: "advisory".into(),
        diff: false,
        ci: "github".into(),
        issue_backend: "jsonl".into(),
        ux_qa: false,
        plan_json: None,
        force_generated_adapters: false,
    })
    .unwrap_err();
    let msg = format!("{err:#}");
    assert!(msg.contains("unknown init profile"), "{msg}");
}

#[test]
fn init_plan_paths_match_profile_manifest() {
    let dir = tempdir().unwrap();
    let plan_path = dir.path().join("plan.json");
    init::run(init::InitArgs {
        repo: dir.path().to_path_buf(),
        apply: false,
        dry_run: true,
        yes: false,
        profile: "rust-ts-postgres".into(),
        ide: "all".into(),
        mode: "advisory".into(),
        diff: false,
        ci: "github".into(),
        issue_backend: "jsonl".into(),
        ux_qa: false,
        plan_json: Some(plan_path.to_string_lossy().into_owned()),
        force_generated_adapters: false,
    })
    .unwrap();

    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&plan_path).unwrap()).unwrap();
    let mut expected: Vec<_> = value["profile_manifest"]["generated_paths"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p.as_str().map(String::from))
        .collect();
    expected.sort();

    let create_paths: Vec<_> = value["actions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| {
            if a["action"] == "create" {
                a["path"].as_str().map(String::from)
            } else {
                None
            }
        })
        .collect();

    let mut sorted_create = create_paths.clone();
    sorted_create.sort();
    assert_eq!(
        sorted_create, expected,
        "every generated path should be a create action when missing"
    );
}

#[test]
fn init_greenfield_apply_then_audit_and_doctor() {
    let dir = tempdir().unwrap();
    init::run(init::InitArgs {
        repo: dir.path().to_path_buf(),
        apply: false,
        dry_run: false,
        yes: true,
        profile: "rust-ts-postgres".into(),
        ide: "all".into(),
        mode: "advisory".into(),
        diff: false,
        ci: "github".into(),
        issue_backend: "jsonl".into(),
        ux_qa: false,
        plan_json: None,
        force_generated_adapters: false,
    })
    .unwrap();

    assert!(dir.path().join("contracts/README.md").exists());
    assert!(dir.path().join("tools/security-lane.sh").exists());
    assert!(dir.path().join("db/README.md").exists());

    let json = dir.path().join("agent/repo-score.json");
    let md = dir.path().join("agent/repo-score.md");
    assert!(Command::new(binary_path())
        .arg("audit")
        .arg(dir.path())
        .arg("--json")
        .arg(&json)
        .arg("--md")
        .arg(&md)
        .status()
        .unwrap()
        .success());

    assert!(Command::new(binary_path())
        .arg("doctor")
        .arg(dir.path())
        .arg("--fail-on")
        .arg("high")
        .status()
        .unwrap()
        .success());
}

#[test]
fn init_respects_existing_contracts_readme() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("contracts")).unwrap();
    fs::write(
        dir.path().join("contracts/README.md"),
        "# Our contracts\nlegacy line\n",
    )
    .unwrap();

    init::run(init::InitArgs {
        repo: dir.path().to_path_buf(),
        apply: false,
        dry_run: false,
        yes: true,
        profile: "rust-ts-postgres".into(),
        ide: "all".into(),
        mode: "advisory".into(),
        diff: false,
        ci: "github".into(),
        issue_backend: "jsonl".into(),
        ux_qa: false,
        plan_json: None,
        force_generated_adapters: false,
    })
    .unwrap();

    let text = fs::read_to_string(dir.path().join("contracts/README.md")).unwrap();
    assert!(
        text.contains("legacy line"),
        "existing contracts README must not be overwritten"
    );
    assert!(!text.contains("Put OpenAPI"));
}

#[test]
fn init_greenfield_apply_rust_api_then_audit_and_doctor() {
    let dir = tempdir().unwrap();
    init::run(init::InitArgs {
        repo: dir.path().to_path_buf(),
        apply: false,
        dry_run: false,
        yes: true,
        profile: "rust-api".into(),
        ide: "all".into(),
        mode: "advisory".into(),
        diff: false,
        ci: "github".into(),
        issue_backend: "jsonl".into(),
        ux_qa: false,
        plan_json: None,
        force_generated_adapters: false,
    })
    .unwrap();

    assert!(dir.path().join("contracts/README.md").exists());
    assert!(dir.path().join("tools/security-lane.sh").exists());
    assert!(dir.path().join("db/README.md").exists());
    assert!(
        !dir.path().join("agent/ux-qa.toml").exists(),
        "rust-api does not include UX QA"
    );

    let json = dir.path().join("agent/repo-score.json");
    let md = dir.path().join("agent/repo-score.md");
    assert!(Command::new(binary_path())
        .arg("audit")
        .arg(dir.path())
        .arg("--json")
        .arg(&json)
        .arg("--md")
        .arg(&md)
        .status()
        .unwrap()
        .success());

    assert!(Command::new(binary_path())
        .arg("doctor")
        .arg(dir.path())
        .arg("--fail-on")
        .arg("high")
        .status()
        .unwrap()
        .success());
}

#[test]
fn init_greenfield_apply_react_web_then_audit_and_doctor() {
    let dir = tempdir().unwrap();
    init::run(init::InitArgs {
        repo: dir.path().to_path_buf(),
        apply: false,
        dry_run: false,
        yes: true,
        profile: "react-web".into(),
        ide: "all".into(),
        mode: "advisory".into(),
        diff: false,
        ci: "github".into(),
        issue_backend: "jsonl".into(),
        ux_qa: false,
        plan_json: None,
        force_generated_adapters: false,
    })
    .unwrap();

    assert!(dir.path().join("contracts/README.md").exists());
    assert!(dir.path().join("tools/security-lane.sh").exists());
    assert!(dir.path().join("agent/ux-qa.toml").exists());
    assert!(
        !dir.path().join("db/README.md").exists(),
        "react-web does not include DB path"
    );

    let json = dir.path().join("agent/repo-score.json");
    let md = dir.path().join("agent/repo-score.md");
    assert!(Command::new(binary_path())
        .arg("audit")
        .arg(dir.path())
        .arg("--json")
        .arg(&json)
        .arg("--md")
        .arg(&md)
        .status()
        .unwrap()
        .success());

    assert!(Command::new(binary_path())
        .arg("doctor")
        .arg(dir.path())
        .arg("--fail-on")
        .arg("high")
        .status()
        .unwrap()
        .success());
}

#[test]
fn init_greenfield_apply_b2b_saas_then_audit_and_doctor() {
    let dir = tempdir().unwrap();
    init::run(init::InitArgs {
        repo: dir.path().to_path_buf(),
        apply: false,
        dry_run: false,
        yes: true,
        profile: "b2b-saas".into(),
        ide: "all".into(),
        mode: "advisory".into(),
        diff: false,
        ci: "github".into(),
        issue_backend: "jsonl".into(),
        ux_qa: false,
        plan_json: None,
        force_generated_adapters: false,
    })
    .unwrap();

    assert!(dir.path().join("contracts/README.md").exists());
    assert!(dir.path().join("tools/security-lane.sh").exists());
    assert!(dir.path().join("agent/ux-qa.toml").exists());
    assert!(dir.path().join("db/README.md").exists());
    assert!(
        dir.path().join("docs/auth/README.md").exists(),
        "b2b-saas includes auth docs"
    );
    assert!(
        dir.path().join("docs/orgs/README.md").exists(),
        "b2b-saas includes orgs docs"
    );

    let json = dir.path().join("agent/repo-score.json");
    let md = dir.path().join("agent/repo-score.md");
    assert!(Command::new(binary_path())
        .arg("audit")
        .arg(dir.path())
        .arg("--json")
        .arg(&json)
        .arg("--md")
        .arg(&md)
        .status()
        .unwrap()
        .success());

    assert!(Command::new(binary_path())
        .arg("doctor")
        .arg(dir.path())
        .arg("--fail-on")
        .arg("high")
        .status()
        .unwrap()
        .success());
}
