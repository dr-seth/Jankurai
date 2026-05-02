use humanlint::versions::check_versions;
use std::fs;
use tempfile::tempdir;

#[test]
fn versions_bindings_validate() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("VERSION"), "0.4.0\n").unwrap();
    fs::create_dir_all(dir.path().join("crates/humanlint")).unwrap();
    fs::write(
        dir.path().join("crates/humanlint/Cargo.toml"),
        "[package]\nname = \"humanlint\"\nversion = \"0.4.0\"\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/standard-version.toml"),
        r#"
standard = "humanlint"
standard_version = "0.4.0"
paper_edition = "2026.05-ed3"
auditor_version = "0.4.0"
schema_version = "1.2.0"
target_stack = "rust-ts-vite-react-postgres-bounded-python"

[[artifact]]
id = "paper-source"
path = "paper/humanlint.tex"
version_field = "paper_edition"
version = "2026.05-ed3"

[[artifact]]
id = "paper-render"
path = "paper/humanlint.pdf"
version_field = "paper_edition"
version = "2026.05-ed3"

[[artifact]]
id = "paper-agent-md"
path = "paper/humanlint.md"
version_field = "paper_edition"
version = "2026.05-ed3"

[[artifact]]
id = "coding-standard"
path = "docs/agent-native-standard.md"
version_field = "standard_version"
version = "0.4.0"

[[artifact]]
id = "agent-standard-brief"
path = "agent/HUMANLINT_STANDARD.md"
version_field = "standard_version"
version = "0.4.0"

[[artifact]]
id = "ux-qa-runtime"
path = "packages/ux-qa"
version_field = "auditor_version"
version = "0.4.0"
"#,
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("paper")).unwrap();
    fs::write(
        dir.path().join("paper/humanlint.md"),
        "Paper edition: `2026.05-ed3`\nStandard version: `0.4.0`\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("paper/humanlint.tex"),
        "\\input{paper/tex/frontmatter}\n",
    )
    .unwrap();
    fs::write(dir.path().join("paper/humanlint.pdf"), "").unwrap();
    fs::create_dir_all(dir.path().join("docs")).unwrap();
    fs::write(
        dir.path().join("docs/agent-native-standard.md"),
        "Standard version: `0.4.0`\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("agent/HUMANLINT_STANDARD.md"),
        "Standard version: `0.4.0`\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("packages/ux-qa")).unwrap();
    fs::write(
        dir.path().join("packages/ux-qa/package.json"),
        "{\n  \"name\": \"@humanlint/ux-qa\",\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();

    check_versions(dir.path()).unwrap();
}
