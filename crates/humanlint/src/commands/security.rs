use crate::model::STANDARD_VERSION;
use crate::validation::{self, ArtifactSchema};
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct SecurityRunArgs {
    pub repo: PathBuf,
    pub script: String,
    pub out: String,
    pub strict: bool,
}

#[derive(Debug, Serialize)]
struct SecurityWrapper {
    kind: String,
    path: String,
    strict: bool,
}

#[derive(Debug, Serialize)]
struct SecurityLaneStep {
    label: String,
    shell_command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool: Option<String>,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    exit_code: Option<i32>,
    advisory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stderr_excerpt: Option<String>,
}

#[derive(Debug, Serialize)]
struct SecurityEvidence {
    schema_version: String,
    standard_version: String,
    generated_at: String,
    repo_root: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    git_head: Option<String>,
    lane: String,
    wrapper: SecurityWrapper,
    exit_code: i32,
    elapsed_ms: u64,
    log_path: String,
    commands: Vec<SecurityLaneStep>,
}

pub fn run(args: SecurityRunArgs) -> Result<()> {
    let repo = args.repo;
    let script_rel = args.script.replace('\\', "/");
    let script_path = repo.join(&script_rel);
    if !script_path.is_file() {
        anyhow::bail!(
            "security lane script `{}` does not exist in {}",
            script_rel,
            repo.display()
        );
    }

    let security_dir = repo.join("target/humanlint/security");
    fs::create_dir_all(&security_dir)?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let log_name = format!(
        "security-lane-{}-{:03}.log",
        ts.as_secs(),
        ts.subsec_millis()
    );
    let log_abs = security_dir.join(&log_name);
    let log_rel = display_rel(&repo, &log_abs);

    let shell_command = format!("bash {}", script_rel);
    let started = Instant::now();

    let mut cmd = Command::new("bash");
    cmd.arg("-lc").arg(&shell_command).current_dir(&repo);
    if args.strict {
        cmd.env("HUMANLINT_SECURITY_STRICT", "1");
    } else {
        cmd.env_remove("HUMANLINT_SECURITY_STRICT");
    }

    let output = cmd
        .output()
        .with_context(|| format!("run `{shell_command}`"))?;

    let exit_code = output.status.code().unwrap_or(-1);
    let mut log_text = String::new();
    log_text.push_str(&format!(
        "command: {shell_command}\nstatus: {:?}\n\n",
        output.status
    ));
    log_text.push_str(&String::from_utf8_lossy(&output.stdout));
    if !output.stdout.is_empty() && !output.stdout.ends_with(b"\n") {
        log_text.push('\n');
    }
    if !output.stderr.is_empty() {
        log_text.push_str("\n[stderr]\n");
        log_text.push_str(&String::from_utf8_lossy(&output.stderr));
        if !output.stderr.ends_with(b"\n") {
            log_text.push('\n');
        }
    }
    fs::write(&log_abs, log_text)?;

    let stderr_lossy = String::from_utf8_lossy(&output.stderr);
    let stderr_excerpt = if exit_code != 0 && !output.stderr.is_empty() {
        Some(cap_excerpt(&stderr_lossy, 500))
    } else {
        None
    };

    let step_status = if exit_code == 0 { "ran" } else { "failed" };
    let generated_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();

    let evidence = SecurityEvidence {
        schema_version: "1.0.0".to_string(),
        standard_version: STANDARD_VERSION.to_string(),
        generated_at,
        repo_root: repo.display().to_string(),
        git_head: git_head(&repo),
        lane: "security".to_string(),
        wrapper: SecurityWrapper {
            kind: "bash_script".to_string(),
            path: script_rel.clone(),
            strict: args.strict,
        },
        exit_code,
        elapsed_ms: started.elapsed().as_millis() as u64,
        log_path: log_rel,
        commands: vec![SecurityLaneStep {
            label: "security-lane".to_string(),
            shell_command,
            tool: Some("bash".to_string()),
            status: step_status.to_string(),
            exit_code: Some(exit_code),
            advisory: false,
            stderr_excerpt,
        }],
    };

    validation::write_json(
        &repo,
        ArtifactSchema::SecurityEvidence,
        &args.out,
        &evidence,
    )?;

    if exit_code != 0 {
        anyhow::bail!("security lane exited with status {exit_code}");
    }

    Ok(())
}

fn cap_excerpt(s: &str, max: usize) -> String {
    let t = s.trim();
    if t.len() <= max {
        t.to_string()
    } else {
        t[t.len().saturating_sub(max)..].to_string()
    }
}

fn git_head(repo: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn display_rel(repo: &Path, path: &Path) -> String {
    path.strip_prefix(repo)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
