use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

pub const GENERATED_MARKER: &str = "humanlint generated adapter";

pub const SUPPORTED_IDES: &[&str] = &[
    "cursor",
    "copilot",
    "claude",
    "gemini",
    "antigravity",
    "aider",
];

pub fn canonical_pointer() -> &'static str {
    "Use `agent/HUMANLINT_STANDARD.md` as the canonical humanlint standard."
}

pub fn master_plan_pointer() -> &'static str {
    "agent/MASTER_PLAN.md"
}

pub fn planner_protocol_pointer() -> &'static str {
    "agent/MASTER_PLAN.md#detailed-planner-protocol"
}

pub const ADAPTER_PATHS: &[&str] = &[
    "CLAUDE.md",
    "GEMINI.md",
    ".cursor/rules/humanlint.mdc",
    ".github/copilot-instructions.md",
    ".github/instructions/humanlint.instructions.md",
    ".github/instructions/humanlint-rust.instructions.md",
    ".github/instructions/humanlint-web.instructions.md",
    ".github/instructions/humanlint-python-ai.instructions.md",
    ".agents/agents.md",
    ".agents/skills/humanlint/SKILL.md",
    ".agents/workflows/humanlint-audit.md",
    ".claude/skills/humanlint/SKILL.md",
];

#[derive(Debug, Clone, Serialize)]
pub struct AdapterAction {
    pub path: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdapterFailure {
    pub path: String,
    pub problem: String,
}

pub fn is_adapter_path(path: &str) -> bool {
    ADAPTER_PATHS.contains(&path)
}

pub fn adapter_plan(repo: &Path, ide: &str) -> Vec<AdapterAction> {
    selected_adapter_paths(ide)
        .into_iter()
        .map(|path| AdapterAction {
            path: path.into(),
            action: if repo.join(path).exists() {
                "keep-existing".into()
            } else {
                "create".into()
            },
        })
        .collect()
}

pub fn write_adapters(repo: &Path, ide: &str, force_generated: bool) -> Result<Vec<AdapterAction>> {
    let mut actions = Vec::new();
    for template in crate::init::templates::TEMPLATES
        .iter()
        .filter(|template| selected_adapter_paths(ide).contains(&template.path))
    {
        let path = repo.join(template.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        if path.exists() {
            let text = fs::read_to_string(&path).unwrap_or_default();
            if force_generated && text.contains(GENERATED_MARKER) {
                fs::write(&path, template.body)
                    .with_context(|| format!("write {}", path.display()))?;
                actions.push(AdapterAction {
                    path: template.path.into(),
                    action: "overwrote-generated".into(),
                });
            } else {
                actions.push(AdapterAction {
                    path: template.path.into(),
                    action: "keep-existing".into(),
                });
            }
            continue;
        }
        fs::write(&path, template.body).with_context(|| format!("write {}", path.display()))?;
        actions.push(AdapterAction {
            path: template.path.into(),
            action: "created".into(),
        });
    }
    Ok(actions)
}

pub fn verify_adapters(repo: &Path) -> Result<Vec<AdapterFailure>> {
    let mut failures = Vec::new();
    for path in ADAPTER_PATHS {
        let full = repo.join(path);
        if !full.exists() {
            continue;
        }
        let text = fs::read_to_string(&full).with_context(|| format!("read {}", full.display()))?;
        if !text.contains("AGENTS.md")
            || !text.contains("agent/HUMANLINT_STANDARD.md")
            || !text.contains(master_plan_pointer())
            || !text.contains(planner_protocol_pointer())
            || !text.contains("tips/phases/00-phase-index.md")
            || !text.contains("tips/phases/logs/")
        {
            failures.push(AdapterFailure {
                path: (*path).into(),
                problem:
                    "adapter lacks canonical AGENTS.md, standard, MASTER_PLAN, planner protocol, phase index, and phase log pointers"
                        .into(),
            });
        }
        let lower = text.to_ascii_lowercase();
        if lower.contains("ignore agents.md")
            || lower.contains("ignore `agents.md`")
            || lower.contains("do not read agents.md")
            || lower.contains("do not use agent/humanlint_standard.md")
            || lower.contains("do not use agent/master_plan.md")
        {
            failures.push(AdapterFailure {
                path: (*path).into(),
                problem: "adapter contains contradictory command/path policy".into(),
            });
        }
        if text.lines().count() > 80 {
            failures.push(AdapterFailure {
                path: (*path).into(),
                problem: "adapter is too long; adapters must not duplicate the full standard"
                    .into(),
            });
        }
    }
    Ok(failures)
}

fn selected_adapter_paths(ide: &str) -> Vec<&'static str> {
    let mut paths = Vec::new();
    for token in ide.split(',').map(str::trim) {
        match token {
            "all" => {
                paths.extend_from_slice(ADAPTER_PATHS);
                break;
            }
            "claude" => {
                paths.push("CLAUDE.md");
                paths.push(".claude/skills/humanlint/SKILL.md");
            }
            "cursor" => paths.push(".cursor/rules/humanlint.mdc"),
            "copilot" => {
                paths.push(".github/copilot-instructions.md");
                paths.push(".github/instructions/humanlint.instructions.md");
                paths.push(".github/instructions/humanlint-rust.instructions.md");
                paths.push(".github/instructions/humanlint-web.instructions.md");
                paths.push(".github/instructions/humanlint-python-ai.instructions.md");
            }
            "gemini" => paths.push("GEMINI.md"),
            "antigravity" => {
                paths.push(".agents/agents.md");
                paths.push(".agents/skills/humanlint/SKILL.md");
                paths.push(".agents/workflows/humanlint-audit.md");
            }
            _ => {}
        }
    }
    paths.sort_unstable();
    paths.dedup();
    paths
}
