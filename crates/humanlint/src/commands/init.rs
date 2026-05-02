use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::validation::{self, ArtifactSchema};

pub struct InitArgs {
    pub repo: PathBuf,
    pub apply: bool,
    pub dry_run: bool,
    pub yes: bool,
    pub profile: String,
    pub ide: String,
    pub mode: String,
    pub diff: bool,
    pub ci: String,
    pub issue_backend: String,
    pub ux_qa: bool,
    pub plan_json: Option<String>,
    pub force_generated_adapters: bool,
}

pub fn run(args: InitArgs) -> Result<()> {
    let plan = crate::init::plan::build_plan(
        &args.repo,
        &args.profile,
        &args.ide,
        &args.mode,
        &args.ci,
        &args.issue_backend,
        args.ux_qa,
    )?;
    if let Some(path) = args.plan_json.as_deref() {
        if args.dry_run || args.diff {
            if let Some(parent) = Path::new(path).parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, serde_json::to_string_pretty(&plan)?)?;
        } else {
            bail!("--plan-json only writes during --dry-run or --diff");
        }
    }
    println!("{}", crate::init::plan::render_plan(&plan));
    if args.dry_run || args.diff {
        if args.diff {
            print_diff(&args.repo, &plan.profile_manifest);
        }
        return Ok(());
    }
    if !(args.apply || args.yes) {
        bail!("refusing to write without --yes or --apply");
    }
    let actions = apply_templates(
        &args.repo,
        &plan.profile_manifest,
        args.force_generated_adapters,
    )?;
    write_receipt(&args.repo, "init", &actions)?;
    Ok(())
}

fn apply_templates(
    repo: &Path,
    manifest: &crate::init::profiles::ProfileManifest,
    force_generated_adapters: bool,
) -> Result<Vec<InitAction>> {
    let mut paths = manifest.generated_paths.clone();
    paths.sort();
    let mut actions = vec![];
    for rel in paths {
        let template = crate::init::templates::template_for_path(&rel)
            .with_context(|| format!("no template registered for profile path `{rel}`"))?;
        let path = repo.join(&rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        if path.exists() {
            if force_generated_adapters
                && crate::init::adapters::is_adapter_path(&rel)
                && fs::read_to_string(&path)
                    .unwrap_or_default()
                    .contains(crate::init::adapters::GENERATED_MARKER)
            {
                fs::write(&path, template.body)
                    .with_context(|| format!("write {}", path.display()))?;
                actions.push(InitAction {
                    path: rel,
                    action: "overwrote-generated-adapter".into(),
                });
                continue;
            }
            if should_mark_for_merge(&rel) {
                let text = fs::read_to_string(&path).unwrap_or_default();
                if is_humanlint_controlled(&text) {
                    actions.push(InitAction {
                        path: rel,
                        action: "kept-existing".into(),
                    });
                } else {
                    append_merge_marker(&path, &rel, &text)?;
                    actions.push(InitAction {
                        path: rel,
                        action: "merge-marker".into(),
                    });
                }
            } else {
                actions.push(InitAction {
                    path: rel,
                    action: "kept-existing".into(),
                });
            }
            continue;
        }
        fs::write(&path, template.body).with_context(|| format!("write {}", path.display()))?;
        actions.push(InitAction {
            path: rel,
            action: "created".into(),
        });
    }
    Ok(actions)
}

fn print_diff(repo: &Path, manifest: &crate::init::profiles::ProfileManifest) {
    let mut paths = manifest.generated_paths.clone();
    paths.sort();
    for rel in paths {
        let Some(template) = crate::init::templates::template_for_path(&rel) else {
            println!("--- {} missing template", rel);
            continue;
        };
        if repo.join(&rel).exists() {
            println!("--- {} exists; no overwrite", rel);
        } else {
            println!("--- /dev/null");
            println!("+++ {}", rel);
            for line in template.body.lines() {
                println!("+{line}");
            }
        }
    }
}

fn should_mark_for_merge(path: &str) -> bool {
    matches!(path, "AGENTS.md" | "agent/HUMANLINT_STANDARD.md")
}

fn is_humanlint_controlled(text: &str) -> bool {
    text.contains("agent/HUMANLINT_STANDARD.md")
        || text.contains("humanlint Standard Agent Bootstrap")
        || (text.contains("Standard version:") && text.to_ascii_lowercase().contains("humanlint"))
}

fn append_merge_marker(path: &Path, rel: &str, text: &str) -> Result<()> {
    let marker = crate::init::merge::merge_marker(rel);
    if text.contains("humanlint merge marker") {
        return Ok(());
    }
    fs::write(path, format!("{text}{marker}"))?;
    Ok(())
}

#[derive(Debug, Serialize)]
struct InitAction {
    path: String,
    action: String,
}

fn write_receipt(repo: &Path, action: &str, actions: &[InitAction]) -> Result<()> {
    let dir = repo.join("target/humanlint/receipts");
    fs::create_dir_all(&dir)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let path = dir.join(format!("{action}-{now}.json"));
    let payload = serde_json::json!({
        "action": action,
        "created_at": now,
        "actions": actions,
    });
    validation::validate_value(repo, ArtifactSchema::InitReceipt, &payload)?;
    fs::write(path, serde_json::to_string_pretty(&payload)?)?;
    Ok(())
}
