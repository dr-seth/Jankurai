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
        run: cargo run -p jankurai -- security run . --out target/jankurai/security/evidence.json
      - name: Phase 12 public evidence
        run: |
          mkdir -p target/jankurai/public
          cargo run -p jankurai -- bench . --out target/jankurai/p12-benchmark-report.json --md target/jankurai/p12-benchmark-report.md
          cargo run -p jankurai -- certify . --out target/jankurai/p12-certification.json --md target/jankurai/p12-certification.md
          cargo run -p jankurai -- govern . --out target/jankurai/p12-governance-policy.json --md target/jankurai/p12-governance-policy.md
          cargo run -p jankurai -- publish . --certification target/jankurai/p12-certification.json --benchmark target/jankurai/p12-benchmark-report.json --governance target/jankurai/p12-governance-policy.json --out target/jankurai/public/p12-public-evidence.json --md target/jankurai/public/p12-public-evidence.md --badge-json target/jankurai/public/jankurai-badge.json --badge-svg target/jankurai/public/jankurai-badge.svg
      - name: Phase 13 optimization and exception expiry
        run: |
          mkdir -p target/jankurai
          cargo run -p jankurai -- optimize . --mode all --out target/jankurai/p13-optimization-report.json --md target/jankurai/p13-optimization-report.md
          cargo run -p jankurai -- exceptions expire . --warning-days 7 --strict --out target/jankurai/p13-exception-expiry.json --md target/jankurai/p13-exception-expiry.md
      - uses: actions/upload-artifact@v4
        with:
          name: jankurai-score
          path: |
            agent/repo-score.json
            agent/repo-score.md
            target/jankurai/jankurai.sarif
            target/jankurai/repair-queue.jsonl
            target/jankurai/security/evidence.json
            target/jankurai/p12-benchmark-report.json
            target/jankurai/p12-benchmark-report.md
            target/jankurai/p12-certification.json
            target/jankurai/p12-certification.md
            target/jankurai/p12-governance-policy.json
            target/jankurai/p12-governance-policy.md
            target/jankurai/public/p12-public-evidence.json
            target/jankurai/public/p12-public-evidence.md
            target/jankurai/public/jankurai-badge.json
            target/jankurai/public/jankurai-badge.svg
            target/jankurai/p13-optimization-report.json
            target/jankurai/p13-optimization-report.md
            target/jankurai/p13-exception-expiry.json
            target/jankurai/p13-exception-expiry.md
"#
    )
}

#[cfg(test)]
mod tests {
    use super::workflow;

    #[test]
    fn workflow_runs_security_via_jankurai() {
        let rendered = workflow("ratchet", 85);
        assert!(rendered.contains("security run"));
        assert!(rendered.contains("target/jankurai/security/evidence.json"));
        assert!(rendered.contains("--mode ratchet"));
        assert!(rendered.contains("-ge 85"));
    }

    #[test]
    fn workflow_includes_phase12_publish_and_badges() {
        let rendered = workflow("ratchet", 85);
        assert!(rendered.contains("Phase 12 public evidence"));
        assert!(rendered.contains("jankurai -- publish"));
        assert!(rendered.contains("target/jankurai/public/p12-public-evidence.json"));
        assert!(rendered.contains("target/jankurai/public/jankurai-badge.svg"));
    }

    #[test]
    fn workflow_includes_phase13_optimize_and_strict_exception_expiry() {
        let rendered = workflow("ratchet", 85);
        assert!(rendered.contains("Phase 13 optimization and exception expiry"));
        assert!(rendered.contains("jankurai -- optimize"));
        assert!(rendered.contains("exceptions expire"));
        assert!(rendered.contains("--strict"));
        assert!(rendered.contains("target/jankurai/p13-exception-expiry.json"));
    }
}
