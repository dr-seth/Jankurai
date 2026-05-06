use jankurai::versions::check_versions;
use std::fs;
use tempfile::tempdir;

#[test]
fn versions_bindings_validate() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("VERSION"), "0.8.9\n").unwrap();
    fs::create_dir_all(dir.path().join("crates/jankurai")).unwrap();
    fs::write(
        dir.path().join("crates/jankurai/Cargo.toml"),
        "[package]\nname = \"jankurai\"\nversion = \"0.8.9\"\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("agent")).unwrap();
    fs::write(
        dir.path().join("agent/standard-version.toml"),
        r#"
standard = "jankurai"
standard_version = "0.8.0"
paper_edition = "2026.05-ed8"
auditor_version = "0.8.9"
schema_version = "1.5.0"
target_stack = "rust-ts-vite-react-postgres-bounded-python"

[[artifact]]
id = "paper-source"
path = "paper/jankurai.tex"
version_field = "paper_edition"
version = "2026.05-ed8"

[[artifact]]
id = "paper-render"
path = "paper/jankurai.pdf"
version_field = "paper_edition"
version = "2026.05-ed8"

[[artifact]]
id = "paper-agent-md"
path = "paper/jankurai.md"
version_field = "paper_edition"
version = "2026.05-ed8"

[[artifact]]
id = "coding-standard"
path = "docs/agent-native-standard.md"
version_field = "standard_version"
version = "0.8.0"

[[artifact]]
id = "agent-standard-brief"
path = "agent/JANKURAI_STANDARD.md"
version_field = "standard_version"
version = "0.8.0"

[[artifact]]
id = "ux-qa-runtime"
path = "packages/ux-qa"
version_field = "auditor_version"
version = "0.8.9"
"#,
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("paper")).unwrap();
    fs::write(
        dir.path().join("paper/jankurai.md"),
        "Paper edition: `2026.05-ed8`\nStandard version: `0.8.0`\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("paper/jankurai.tex"),
        "\\input{paper/tex/frontmatter}\n",
    )
    .unwrap();
    fs::write(dir.path().join("paper/jankurai.pdf"), "").unwrap();
    fs::create_dir_all(dir.path().join("docs")).unwrap();
    fs::write(
        dir.path().join("docs/agent-native-standard.md"),
        "Standard version: `0.8.0`\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("agent/JANKURAI_STANDARD.md"),
        "Standard version: `0.8.0`\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("packages/ux-qa")).unwrap();
    fs::write(
        dir.path().join("packages/ux-qa/package.json"),
        "{\n  \"name\": \"@jankurai/ux-qa\",\n  \"version\": \"0.8.9\"\n}\n",
    )
    .unwrap();

    check_versions(dir.path()).unwrap();
}
