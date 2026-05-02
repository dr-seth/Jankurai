use jankurai::commands::init;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn init_dry_run_writes_nothing() {
    let dir = tempdir().unwrap();
    init::run(init::InitArgs {
        repo: dir.path().to_path_buf(),
        apply: false,
        dry_run: true,
        yes: false,
        profile: "rust-ts-vite-react-postgres".into(),
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

    assert!(!dir.path().join("AGENTS.md").exists());
    assert!(!dir.path().join("target/jankurai/receipts").exists());
}

#[test]
fn init_yes_is_idempotent_for_existing_root_guidance() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "# Existing\n").unwrap();

    for _ in 0..2 {
        init::run(init::InitArgs {
            repo: dir.path().to_path_buf(),
            apply: false,
            dry_run: false,
            yes: true,
            profile: "rust-ts-vite-react-postgres".into(),
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
    }

    let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert_eq!(agents.matches("jankurai merge marker").count(), 1);
    assert!(dir.path().join(".cursor/rules/jankurai.mdc").exists());
    assert!(dir.path().join("CLAUDE.md").exists());
    assert!(dir.path().join("GEMINI.md").exists());
    assert!(dir.path().join(".github/copilot-instructions.md").exists());
    assert!(dir.path().join(".agents/agents.md").exists());
    assert!(dir.path().join("target/jankurai/receipts").exists());
    for rel in [
        ".cursor/rules/jankurai.mdc",
        "CLAUDE.md",
        "GEMINI.md",
        ".github/copilot-instructions.md",
        ".agents/agents.md",
    ] {
        let text = fs::read_to_string(dir.path().join(rel)).unwrap();
        assert!(
            text.contains("agent/MASTER_PLAN.md#detailed-planner-protocol"),
            "generated adapter {rel} should point at detailed planner protocol"
        );
    }
}

#[test]
fn init_dry_run_plan_json_is_machine_readable() {
    let dir = tempdir().unwrap();
    let plan = dir.path().join("target/jankurai/init-plan.json");

    init::run(init::InitArgs {
        repo: dir.path().to_path_buf(),
        apply: false,
        dry_run: true,
        yes: false,
        profile: "rust-ts-vite-react-postgres-bounded-python".into(),
        ide: "all".into(),
        mode: "advisory".into(),
        diff: false,
        ci: "github".into(),
        issue_backend: "jsonl".into(),
        ux_qa: true,
        plan_json: Some(plan.display().to_string()),
        force_generated_adapters: false,
    })
    .unwrap();

    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(plan).unwrap()).unwrap();
    assert_eq!(
        value["profile"],
        "rust-ts-vite-react-postgres-bounded-python"
    );
    assert!(value["actions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|action| { action["path"] == "CLAUDE.md" && action["action"] == "create" }));
    assert!(!dir.path().join("AGENTS.md").exists());
}

#[test]
fn init_dry_run_profile_manifest_is_included() {
    let dir = tempdir().unwrap();
    let plan = dir.path().join("target/jankurai/init-plan.json");

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
        ux_qa: true,
        plan_json: Some(plan.display().to_string()),
        force_generated_adapters: false,
    })
    .unwrap();

    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(plan).unwrap()).unwrap();
    assert_eq!(value["profile"], "rust-ts-postgres");
    assert_eq!(value["profile_manifest"]["id"], "rust-ts-postgres");
    assert_eq!(
        value["profile_manifest"]["target_stack_id"],
        "rust-ts-vite-react-postgres-bounded-python"
    );
    assert!(value["profile_manifest"]["generated_paths"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path == "agent/ux-qa.toml"));
    assert!(value["profile_manifest"]["validation_commands"]
        .as_array()
        .unwrap()
        .iter()
        .any(|cmd| cmd == "just fast"));
}

#[test]
fn init_yes_keeps_existing_jankurai_guidance_without_marker() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("AGENTS.md"),
        "Read `agent/JANKURAI_STANDARD.md` first.\n",
    )
    .unwrap();

    init::run(init::InitArgs {
        repo: dir.path().to_path_buf(),
        apply: false,
        dry_run: false,
        yes: true,
        profile: "rust-ts-vite-react-postgres".into(),
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

    let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(!agents.contains("jankurai merge marker"));
}

#[test]
fn cli_surfaces_smoke() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "Read agent standard\n").unwrap();
    fs::write(dir.path().join("README.md"), "# Repo\n").unwrap();
    fs::write(dir.path().join("Justfile"), "fast:\n    cargo test\n").unwrap();
    fs::create_dir_all(dir.path().join("db")).unwrap();
    fs::write(dir.path().join("db/README.md"), "# db\n").unwrap();
    fs::create_dir_all(dir.path().join("tools")).unwrap();
    fs::write(
        dir.path().join("tools/security-lane.sh"),
        "#!/bin/sh\nexit 0\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/boundaries.toml"),
        r#"
[stack]
id = "test-stack"
[queues]
adapter_paths = []
event_contract_paths = []
generated_type_paths = []
"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("agent/security-policy.toml"),
        r#"
schema_version = "1.0.0"
enabled_tools = ["gitleaks"]
required_tools = []
advisory_tools = ["gitleaks"]
"#,
    )
    .unwrap();
    for rel in [
        "JANKURAI_STANDARD.md",
        "owner-map.json",
        "test-map.json",
        "generated-zones.toml",
        "proof-lanes.toml",
        "standard-version.toml",
        "repo-score.json",
        "repo-score.md",
    ] {
        fs::write(dir.path().join("agent").join(rel), "{}\n").unwrap();
    }

    let json = dir.path().join("score.json");
    let md = dir.path().join("score.md");
    assert!(Command::new(env!("CARGO_BIN_EXE_jankurai"))
        .arg("audit")
        .arg(dir.path())
        .arg("--json")
        .arg(&json)
        .arg("--md")
        .arg(&md)
        .status()
        .unwrap()
        .success());
    assert!(json.exists());
    assert!(md.exists());

    assert!(Command::new(env!("CARGO_BIN_EXE_jankurai"))
        .arg("doctor")
        .arg(dir.path())
        .arg("--fail-on")
        .arg("high")
        .status()
        .unwrap()
        .success());

    let issues = dir.path().join("issues.jsonl");
    assert!(Command::new(env!("CARGO_BIN_EXE_jankurai"))
        .arg("issues")
        .arg("export")
        .arg(dir.path())
        .arg("--format")
        .arg("jsonl")
        .arg("--out")
        .arg(&issues)
        .status()
        .unwrap()
        .success());
    assert!(issues.exists());

    let ci_dir = tempdir().unwrap();
    assert!(Command::new(env!("CARGO_BIN_EXE_jankurai"))
        .arg("ci")
        .arg("install")
        .arg(ci_dir.path())
        .arg("--github")
        .arg("--mode")
        .arg("ratchet")
        .arg("--min-score")
        .arg("85")
        .status()
        .unwrap()
        .success());
    let workflow =
        fs::read_to_string(ci_dir.path().join(".github/workflows/jankurai.yml")).unwrap();
    assert!(workflow.contains("Enforce score floor"));
    assert!(workflow.contains("bash tools/security-lane.sh"));

    assert!(Command::new(env!("CARGO_BIN_EXE_jankurai"))
        .arg("explain")
        .arg("HLT-003-OWNERLESS-PATH")
        .status()
        .unwrap()
        .success());
}
