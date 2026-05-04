use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct CiInstallArgs {
    pub repo: PathBuf,
    pub github: bool,
    pub mode: String,
    pub min_score: i32,
    pub baseline: Option<String>,
    pub dry_run: bool,
}

pub fn install(args: CiInstallArgs) -> Result<()> {
    if !args.github {
        bail!("only `jankurai ci install --github` is currently supported");
    }
    if !matches!(args.mode.as_str(), "observe" | "advisory" | "ratchet") {
        bail!(
            "unknown CI mode `{}`; expected observe, advisory, or ratchet",
            args.mode
        );
    }
    let path = args.repo.join(".github/workflows/jankurai.yml");
    let rendered = workflow(&args.mode, args.min_score, args.baseline.as_deref());
    if args.dry_run {
        println!("# would write {}", path.display());
        print!("{rendered}");
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if path.exists() {
        println!("{}: exists; leaving user content unchanged", path.display());
        return Ok(());
    }
    fs::write(&path, rendered).with_context(|| format!("write {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}

fn workflow(mode: &str, min_score: i32, baseline: Option<&str>) -> String {
    let audit_mode = if mode == "ratchet" {
        "ratchet"
    } else {
        "advisory"
    };
    let baseline_arg = baseline
        .map(|path| format!(" --baseline {path}"))
        .unwrap_or_default();
    let gate = if mode == "ratchet" {
        format!(
            r#"
      - name: Enforce score floor
        run: test "$(jq -r '.score' target/jankurai/repo-score.json)" -ge {min_score}"#
        )
    } else {
        String::new()
    };
    format!(
        r#"name: jankurai

on:
  pull_request:
  push:
    branches: [main]

jobs:
  audit:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install jankurai
        run: cargo install jankurai --locked
      - run: jankurai --version
      - name: jankurai audit
        run: jankurai audit . --mode {audit_mode}{baseline_arg} --json target/jankurai/repo-score.json --md target/jankurai/repo-score.md --sarif target/jankurai/jankurai.sarif --github-step-summary target/jankurai/summary.md --repair-queue-jsonl target/jankurai/repair-queue.jsonl{gate}
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: jankurai-adoption-evidence
          path: |
            target/jankurai/repo-score.json
            target/jankurai/repo-score.md
            target/jankurai/jankurai.sarif
            target/jankurai/repair-queue.jsonl
"#
    )
}

#[cfg(test)]
mod tests {
    use super::workflow;

    #[test]
    fn ratchet_workflow_uses_installed_jankurai_and_score_gate() {
        let rendered = workflow("ratchet", 85, None);
        assert!(rendered.contains("cargo install jankurai --locked"));
        assert!(rendered.contains("--mode ratchet"));
        assert!(rendered.contains("-ge 85"));
    }

    #[test]
    fn observe_workflow_has_no_score_gate() {
        let rendered = workflow("observe", 85, None);
        assert!(rendered.contains("--mode advisory"));
        assert!(!rendered.contains("Enforce score floor"));
        assert!(!rendered.contains("cargo run -p jankurai"));
    }

    #[test]
    fn ratchet_workflow_can_use_baseline() {
        let rendered = workflow("ratchet", 85, Some("target/jankurai/baseline.json"));
        assert!(rendered.contains("--baseline target/jankurai/baseline.json"));
        assert!(rendered.contains("target/jankurai/repo-score.json"));
    }
}
