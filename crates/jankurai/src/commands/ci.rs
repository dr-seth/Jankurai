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
    let progress = crate::ui::CliProgress::new("installing CI workflow", 4);
    progress.tick("validate options");
    if !args.github {
        bail!("only `jankurai ci install --github` is currently supported");
    }
    if !matches!(args.mode.as_str(), "observe" | "advisory" | "ratchet") {
        bail!(
            "unknown CI mode `{}`; expected observe, advisory, or ratchet",
            args.mode
        );
    }
    if args.mode == "ratchet" && args.baseline.is_none() {
        bail!("ratchet CI requires --baseline PATH; install observe/advisory until an accepted baseline exists");
    }
    progress.tick("render workflow");
    let path = args.repo.join(".github/workflows/jankurai.yml");
    let rendered = workflow(&args.mode, args.min_score, args.baseline.as_deref());
    if args.dry_run {
        progress.finish("dry-run workflow rendered");
        println!(
            "{}",
            crate::ui::paint(
                crate::ui::Style::Accent,
                format!("# would write {}", path.display()),
                crate::ui::stdout_color_enabled()
            )
        );
        print!("{rendered}");
        return Ok(());
    }
    progress.tick("prepare workflow directory");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if path.exists() {
        progress.finish("existing workflow preserved");
        println!(
            "{}",
            crate::ui::paint(
                crate::ui::Style::Warn,
                format!("{}: exists; leaving user content unchanged", path.display()),
                crate::ui::stdout_color_enabled()
            )
        );
        return Ok(());
    }
    progress.tick("write workflow");
    fs::write(&path, rendered).with_context(|| format!("write {}", path.display()))?;
    progress.finish("CI workflow installed");
    println!(
        "{}",
        crate::ui::paint(
            crate::ui::Style::Good,
            format!("wrote {}", path.display()),
            crate::ui::stdout_color_enabled()
        )
    );
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
      - name: jankurai merge witness
        run: jankurai witness . --changed-from origin/main{baseline_arg} --out target/jankurai/merge-witness.json --md target/jankurai/merge-witness.md
      - name: jankurai score diff
        run: jankurai score diff --base {baseline} --head target/jankurai/repo-score.json --out target/jankurai/score-diff.json --md target/jankurai/score-diff.md
      - name: Enforce score floor
        run: test "$(jq -r '.score' target/jankurai/repo-score.json)" -ge {min_score}"#,
            baseline = baseline.unwrap_or("agent/repo-score.json")
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
        with:
          fetch-depth: 0
      - uses: dtolnay/rust-toolchain@stable
      - name: Install jankurai
        run: cargo install jankurai --locked
      - run: jankurai --version
      - name: jankurai audit
        run: jankurai audit . --mode {audit_mode}{baseline_arg} --json target/jankurai/repo-score.json --md target/jankurai/repo-score.md --sarif target/jankurai/jankurai.sarif --github-step-summary target/jankurai/summary.md --repair-queue-jsonl target/jankurai/repair-queue.jsonl{gate}
      - name: Proofbind verify
        run: jankurai proofbind verify . --changed-from origin/main
      - name: Proofmark rust
        run: jankurai proofmark rust . --obligations target/jankurai/proofbind/obligations.json
      - name: Rust witness build
        run: jankurai rust witness build .
      - name: UX QA smoke
        run: jankurai ux audit --config agent/ux-qa.toml --out target/jankurai/ux-qa.json
      - name: jankurai badge check
        run: jankurai badge . --check --update-readme
        continue-on-error: true
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: jankurai-adoption-evidence
          if-no-files-found: ignore
          path: |
            target/jankurai/repo-score.json
            target/jankurai/repo-score.md
            target/jankurai/jankurai.sarif
            target/jankurai/repair-queue.jsonl
            target/jankurai/proofbind/obligations.json
            target/jankurai/proofbind/surface-witness.json
            target/jankurai/proofmark/proofmark-receipt.json
            target/jankurai/proofmark/proof-receipt.json
            target/jankurai/rust/witness-graph.json
            target/jankurai/ux-qa.json
            target/jankurai/merge-witness.json
            target/jankurai/merge-witness.md
            target/jankurai/score-diff.json
            target/jankurai/score-trend.json
            target/jankurai/security/evidence.json
            target/jankurai/migration-report.json
            agent/jankurai-badge.svg
            agent/jankurai-badge.json
"#
    )
}

#[cfg(test)]
mod tests {
    use super::workflow;

    #[test]
    fn ratchet_workflow_uses_installed_jankurai_and_score_gate() {
        let rendered = workflow("ratchet", 85, Some("agent/repo-score.json"));
        assert!(rendered.contains("cargo install jankurai --locked"));
        assert!(rendered.contains("--mode ratchet"));
        assert!(rendered.contains("jankurai witness"));
        assert!(rendered.contains("-ge 85"));
    }

    #[test]
    fn observe_workflow_has_no_score_gate() {
        let rendered = workflow("observe", 85, None);
        assert!(rendered.contains("--mode advisory"));
        assert!(!rendered.contains("Enforce score floor"));
        assert!(!rendered.contains("-ge 85"));
    }

    #[test]
    fn ratchet_workflow_can_use_baseline() {
        let rendered = workflow("ratchet", 85, Some("target/jankurai/baseline.json"));
        assert!(rendered.contains("--baseline target/jankurai/baseline.json"));
        assert!(rendered.contains("target/jankurai/repo-score.json"));
    }
}
