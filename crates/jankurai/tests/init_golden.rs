use jankurai::commands::init;
use jankurai::init::profiles::BUNDLED_PROFILE_IDS;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_jankurai")
}

fn dry_run_plan_args(repo: std::path::PathBuf, profile: &str, plan_json: Option<String>) -> init::InitArgs {
    init::InitArgs {
        repo,
        apply: false,
        dry_run: true,
        yes: false,
        profile: profile.into(),
        profile_file: None,
        ide: "all".into(),
        mode: "advisory".into(),
        diff: false,
        ci: "github".into(),
        issue_backend: "jsonl".into(),
        ux_qa: false,
        plan_json,
        force_generated_adapters: false,
    }
}

fn greenfield_apply_args(repo: std::path::PathBuf, profile: &str) -> init::InitArgs {
    init::InitArgs {
        repo,
        apply: false,
        dry_run: false,
        yes: true,
        profile: profile.into(),
        profile_file: None,
        ide: "all".into(),
        mode: "advisory".into(),
        diff: false,
        ci: "github".into(),
        issue_backend: "jsonl".into(),
        ux_qa: false,
        plan_json: None,
        force_generated_adapters: false,
    }
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
        profile_file: None,
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
fn init_plan_paths_match_profile_manifest_for_all_bundled_profiles() {
    for profile in BUNDLED_PROFILE_IDS {
        let dir = tempdir().unwrap();
        let plan_path = dir.path().join(format!("plan-{profile}.json"));
        init::run(dry_run_plan_args(
            dir.path().to_path_buf(),
            profile,
            Some(plan_path.to_string_lossy().into_owned()),
        ))
        .unwrap_or_else(|e| panic!("profile {profile}: {e:#}"));

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
            "profile {profile}: every generated path should be a create action when missing"
        );
    }
}

#[test]
fn init_profile_aliases_resolve_in_plan() {
    let dir = tempdir().unwrap();
    let plan_path = dir.path().join("plan.json");
    init::run(dry_run_plan_args(
        dir.path().to_path_buf(),
        "ai",
        Some(plan_path.to_string_lossy().into_owned()),
    ))
    .unwrap();
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&plan_path).unwrap()).unwrap();
    assert_eq!(value["profile_manifest"]["id"], "ai-product");
}

#[test]
fn init_greenfield_apply_then_audit_and_doctor() {
    let dir = tempdir().unwrap();
    init::run(greenfield_apply_args(
        dir.path().to_path_buf(),
        "rust-ts-postgres",
    ))
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
        profile_file: None,
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
    init::run(greenfield_apply_args(dir.path().to_path_buf(), "rust-api")).unwrap();

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
    init::run(greenfield_apply_args(dir.path().to_path_buf(), "react-web")).unwrap();

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
    init::run(greenfield_apply_args(dir.path().to_path_buf(), "b2b-saas")).unwrap();

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

#[test]
fn init_greenfield_apply_ai_product_then_audit_and_doctor() {
    let dir = tempdir().unwrap();
    init::run(greenfield_apply_args(dir.path().to_path_buf(), "ai-product")).unwrap();

    assert!(dir.path().join("contracts/README.md").exists());
    assert!(dir.path().join("python/ai-service/README.md").exists());
    assert!(dir.path().join("prompts/README.md").exists());
    assert!(
        !dir.path().join("agent/ux-qa.toml").exists(),
        "ai-product omits web UX controls by default"
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
fn init_greenfield_apply_regulated_saas_then_audit_and_doctor() {
    let dir = tempdir().unwrap();
    init::run(greenfield_apply_args(
        dir.path().to_path_buf(),
        "regulated-saas",
    ))
    .unwrap();

    assert!(dir.path().join("docs/privacy/README.md").exists());
    assert!(dir.path().join("docs/compliance/README.md").exists());
    assert!(dir.path().join("agent/ux-qa.toml").exists());

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
fn init_greenfield_apply_migration_target_then_audit_and_doctor() {
    let dir = tempdir().unwrap();
    init::run(greenfield_apply_args(
        dir.path().to_path_buf(),
        "migration-target",
    ))
    .unwrap();

    assert!(dir.path().join("docs/migration/boundary-map.md").exists());
    assert!(
        !dir.path().join("db/README.md").exists(),
        "migration-target does not claim database ownership"
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
fn init_profile_file_loads_manifest_from_disk() {
    let dir = tempdir().unwrap();
    let profile_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates/profiles/rust-api.json");
    let plan_path = dir.path().join("plan.json");
    init::run(init::InitArgs {
        repo: dir.path().to_path_buf(),
        apply: false,
        dry_run: true,
        yes: false,
        profile: "this-value-is-ignored-when-profile-file-is-set".into(),
        profile_file: Some(profile_path),
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
    assert_eq!(value["profile"], "rust-api");
    assert_eq!(value["profile_manifest"]["id"], "rust-api");
}

#[test]
fn init_profile_file_rejects_invalid_manifest() {
    let dir = tempdir().unwrap();
    let bad = dir.path().join("bad-profile.json");
    fs::write(&bad, r#"{"id": "only-id"}"#).unwrap();
    let err = init::run(init::InitArgs {
        repo: dir.path().to_path_buf(),
        apply: false,
        dry_run: true,
        yes: false,
        profile: "rust-api".into(),
        profile_file: Some(bad),
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
    assert!(
        msg.contains("displayName") || msg.contains("required") || msg.contains("schema"),
        "{msg}"
    );
}

#[test]
fn init_merges_existing_json() {
    let dir = tempdir().unwrap();
    let agent_dir = dir.path().join("agent");
    fs::create_dir_all(&agent_dir).unwrap();
    
    // Seed an existing owner-map.json
    fs::write(
        agent_dir.join("owner-map.json"),
        r#"{
  "schema": "https://jankurai.io/schemas/owner-map.schema.json",
  "version": 1,
  "owners": {
    "custom/": "my-custom-agent"
  }
}"#,
    )
    .unwrap();

    init::run(greenfield_apply_args(
        dir.path().to_path_buf(),
        "rust-ts-postgres",
    ))
    .unwrap();

    let json_text = fs::read_to_string(agent_dir.join("owner-map.json")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json_text).unwrap();
    let owners = value["owners"].as_object().unwrap();
    
    let has_custom = owners.get("custom/").map_or(false, |v| v == "my-custom-agent");
    let has_standard = owners.get("crates/").map_or(false, |v| v == "tools");
    
    assert!(has_custom, "must retain existing custom owner");
    assert!(has_standard, "must merge in standard crates owner from template");
}

#[test]
fn init_merges_existing_lines_justfile() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Justfile"),
        "# existing custom recipe\n\ncustom:\n\t@echo preserved\n",
    )
    .unwrap();

    init::run(greenfield_apply_args(
        dir.path().to_path_buf(),
        "rust-ts-postgres",
    ))
    .unwrap();

    let text = fs::read_to_string(dir.path().join("Justfile")).unwrap();
    assert!(
        text.contains("preserved"),
        "must retain existing Justfile content: {text}"
    );
    assert!(
        text.contains("fast:") && text.contains("cargo test -p jankurai"),
        "must merge in scaffold recipes from template: {text}"
    );
}

#[test]
fn init_merges_existing_toml() {
    let dir = tempdir().unwrap();
    let agent_dir = dir.path().join("agent");
    fs::create_dir_all(&agent_dir).unwrap();
    
    // Seed an existing proof-lanes.toml
    fs::write(
        agent_dir.join("proof-lanes.toml"),
        r#"
schema = "https://jankurai.io/schemas/proof-lanes.schema.json"
version = 1

[[lane]]
name = "custom-lane"
command = "echo custom"
"#,
    )
    .unwrap();

    init::run(greenfield_apply_args(
        dir.path().to_path_buf(),
        "rust-ts-postgres",
    ))
    .unwrap();

    let toml_text = fs::read_to_string(agent_dir.join("proof-lanes.toml")).unwrap();
    let value: toml::Value = toml::from_str(&toml_text).unwrap();
    let lanes = value["lane"].as_array().unwrap();
    
    let has_custom = lanes.iter().any(|l| l["name"].as_str() == Some("custom-lane"));
    let has_standard = lanes.iter().any(|l| l["name"].as_str() == Some("fast"));
    
    assert!(has_custom, "must retain existing custom lane");
    assert!(has_standard, "must merge in standard fast lane from template");
}
