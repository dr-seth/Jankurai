use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct CiInstallArgs {
    pub repo: PathBuf,
    pub github: bool,
    pub mode: String,
    pub min_score: i32,
}

pub fn install(args: CiInstallArgs) -> Result<()> {
    if !args.github {
        bail!("only `jankurai ci install --github` is currently supported");
    }
    let path = args.repo.join(".github/workflows/jankurai.yml");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if path.exists() {
        println!("{}: exists; leaving user content unchanged", path.display());
        return Ok(());
    }
    fs::write(&path, workflow(&args.mode, args.min_score))
        .with_context(|| format!("write {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}

fn workflow(mode: &str, min_score: i32) -> String {
    format!(
        r#"name: jankurai

on:
  pull_request:
  push:
    branches: [main]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with:
          node-version: "22"
          cache: npm
      - run: npm ci
      - run: cargo run -p jankurai -- versions
      - name: jankurai audit
        run: cargo run -p jankurai -- audit . --mode {mode} --json agent/repo-score.json --md agent/repo-score.md --sarif target/jankurai/jankurai.sarif --github-step-summary target/jankurai/summary.md --repair-queue-jsonl target/jankurai/repair-queue.jsonl
      - name: Enforce score floor
        run: test "$(jq -r '.score' agent/repo-score.json)" -ge {min_score}
      - name: Security lane
        run: bash tools/security-lane.sh
      - uses: actions/upload-artifact@v4
        with:
          name: jankurai-score
          path: |
            agent/repo-score.json
            agent/repo-score.md
            target/jankurai/jankurai.sarif
            target/jankurai/repair-queue.jsonl
"#
    )
}

#[cfg(test)]
mod tests {
    use super::workflow;

    #[test]
    fn workflow_uses_shared_security_lane_script() {
        let rendered = workflow("ratchet", 85);
        assert!(rendered.contains("bash tools/security-lane.sh"));
        assert!(rendered.contains("--mode ratchet"));
        assert!(rendered.contains("-ge 85"));
    }
}
