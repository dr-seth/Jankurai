use anyhow::{bail, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct InitPlan {
    pub profile: String,
    pub level: String,
    pub profile_manifest: super::profiles::ProfileManifest,
    pub ide: String,
    pub mode: String,
    pub ci: String,
    pub issue_backend: String,
    pub ux_qa: bool,
    pub package_manager: String,
    pub detected: Vec<String>,
    pub actions: Vec<PlannedAction>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedAction {
    pub path: String,
    pub action: String,
}

pub fn build_plan(
    repo: &Path,
    profile: &str,
    profile_file: Option<&Path>,
    level: &str,
    ide: &str,
    mode: &str,
    ci: &str,
    issue_backend: &str,
    ux_qa: bool,
) -> Result<InitPlan> {
    let existing = super::detect::existing_standard_files(repo);
    let detected = super::detect::detect_surfaces(repo);
    let selected_level = InitLevel::parse(level)?;
    let profile_manifest = match profile_file {
        Some(path) => super::profiles::load_profile_from_path(repo, path)?,
        None => super::profiles::resolve_profile(repo, profile)?,
    };
    let profile_manifest = filter_profile_manifest(profile_manifest, selected_level);
    let profile = profile_manifest.id.clone();
    let mut paths = profile_manifest.generated_paths.clone();
    paths.sort();
    let mut actions = Vec::new();
    for path in &paths {
        if super::templates::template_for_path(path).is_none() {
            bail!("profile declares `{path}` but no init template is registered (see init/templates.rs)");
        }
        let action = if repo.join(path).exists() {
            profile_manifest
                .merge_policy_for_path(path)
                .plan_action()
                .into()
        } else {
            "create".into()
        };
        actions.push(PlannedAction {
            path: path.clone(),
            action,
        });
    }
    let mut warnings = Vec::new();
    if !existing.is_empty() {
        warnings.push(format!(
            "existing files left intact: {}",
            existing.join(", ")
        ));
    }
    Ok(InitPlan {
        profile,
        level: selected_level.as_str().into(),
        profile_manifest,
        ide: ide.into(),
        mode: mode.into(),
        ci: ci.into(),
        issue_backend: issue_backend.into(),
        ux_qa,
        package_manager: super::package_manager::detect_package_manager(repo).to_string(),
        detected,
        actions,
        warnings,
    })
}

pub fn render_plan(plan: &InitPlan) -> String {
    let mut out = String::new();
    use std::fmt::Write;
    let _ = writeln!(out, "Jankurai Init Plan");
    let _ = writeln!(out);
    let _ = writeln!(out, "- profile: `{}`", plan.profile);
    let _ = writeln!(out, "- level: `{}`", plan.level);
    let _ = writeln!(out, "- profile id: `{}`", plan.profile_manifest.id);
    let _ = writeln!(
        out,
        "- profile display: `{}`",
        plan.profile_manifest.display_name
    );
    let _ = writeln!(
        out,
        "- target stack: `{}`",
        plan.profile_manifest.target_stack_id
    );
    let _ = writeln!(out, "- ide: `{}`", plan.ide);
    let _ = writeln!(out, "- mode: `{}`", plan.mode);
    let _ = writeln!(out, "- ci: `{}`", plan.ci);
    let _ = writeln!(out, "- issue backend: `{}`", plan.issue_backend);
    let _ = writeln!(out, "- ux qa: `{}`", plan.ux_qa);
    let _ = writeln!(out, "- package manager: `{}`", plan.package_manager);
    if !plan.profile_manifest.required_lanes.is_empty() {
        let _ = writeln!(
            out,
            "- required lanes: `{}`",
            plan.profile_manifest.required_lanes.join(", ")
        );
    }
    if !plan.profile_manifest.optional_lanes.is_empty() {
        let _ = writeln!(
            out,
            "- optional lanes: `{}`",
            plan.profile_manifest.optional_lanes.join(", ")
        );
    }
    if !plan.detected.is_empty() {
        let _ = writeln!(out, "- detected: `{}`", plan.detected.join(", "));
    }
    if !plan.profile_manifest.validation_commands.is_empty() {
        let _ = writeln!(
            out,
            "- validation: `{}`",
            plan.profile_manifest.validation_commands.join(" | ")
        );
    }
    for warning in &plan.warnings {
        let _ = writeln!(out, "- warning: `{warning}`");
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "Planned file actions:");
    for action in &plan.actions {
        let _ = writeln!(out, "  {} {}", action.action, action.path);
    }
    out
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InitLevel {
    Agents,
    Score,
    Ci,
    Full,
}

impl InitLevel {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "agents" => Ok(Self::Agents),
            "score" => Ok(Self::Score),
            "ci" => Ok(Self::Ci),
            "full" => Ok(Self::Full),
            other => bail!("unknown init level `{other}`; expected agents, score, ci, or full"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Agents => "agents",
            Self::Score => "score",
            Self::Ci => "ci",
            Self::Full => "full",
        }
    }
}

fn filter_profile_manifest(
    mut manifest: super::profiles::ProfileManifest,
    level: InitLevel,
) -> super::profiles::ProfileManifest {
    if level == InitLevel::Full {
        return manifest;
    }

    let allowed = level_allowed_paths(level);
    manifest
        .generated_paths
        .retain(|path| allowed.contains(path.as_str()));
    if matches!(level, InitLevel::Score | InitLevel::Ci)
        && !manifest
            .generated_paths
            .iter()
            .any(|path| path == "Justfile")
    {
        manifest.generated_paths.push("Justfile".into());
        manifest.merge_policy.insert(
            "Justfile".into(),
            super::profiles::MergePolicyAction::MergeLines,
        );
    }
    manifest.agent_adapters = filter_list(manifest.agent_adapters, &allowed);
    manifest.ci_templates = filter_list(manifest.ci_templates, &allowed);
    manifest.docs = filter_list(manifest.docs, &allowed);
    manifest.security_controls = filter_list(manifest.security_controls, &allowed);
    manifest.ux_controls = filter_list(manifest.ux_controls, &allowed);
    manifest.contract_system = filter_list(manifest.contract_system, &allowed);
    manifest.db_policy = filter_list(manifest.db_policy, &allowed);
    manifest.required_lanes = level_required_lanes(level);
    manifest.optional_lanes = level_optional_lanes(level);
    manifest.validation_commands = level_validation_commands(level);

    let generated: BTreeSet<&str> = manifest
        .generated_paths
        .iter()
        .map(String::as_str)
        .collect();
    manifest.merge_policy = manifest
        .merge_policy
        .into_iter()
        .filter(|(path, _)| generated.contains(path.as_str()))
        .collect::<BTreeMap<_, _>>();
    manifest
}

fn filter_list(values: Vec<String>, allowed: &BTreeSet<&'static str>) -> Vec<String> {
    values
        .into_iter()
        .filter(|path| allowed.contains(path.as_str()))
        .collect()
}

fn level_allowed_paths(level: InitLevel) -> BTreeSet<&'static str> {
    let mut paths = BTreeSet::from([
        "AGENTS.md",
        "CLAUDE.md",
        "GEMINI.md",
        ".agents/agents.md",
        ".agents/skills/jankurai/SKILL.md",
        ".agents/workflows/jankurai-audit.md",
        ".claude/skills/jankurai/SKILL.md",
        ".cursor/rules/jankurai.mdc",
        ".github/copilot-instructions.md",
        ".github/instructions/jankurai.instructions.md",
        ".github/instructions/jankurai-python-ai.instructions.md",
        ".github/instructions/jankurai-rust.instructions.md",
        ".github/instructions/jankurai-web.instructions.md",
        "agent/JANKURAI_STANDARD.md",
        "agent/MASTER_PLAN.md",
    ]);

    if matches!(level, InitLevel::Score | InitLevel::Ci) {
        paths.extend([
            "Justfile",
            "agent/audit-policy.toml",
            "agent/generated-zones.toml",
            "agent/owner-map.json",
            "agent/proof-lanes.toml",
            "agent/standard-version.toml",
            "agent/test-map.json",
        ]);
    }

    if level == InitLevel::Ci {
        paths.extend([
            ".github/workflows/jankurai.yml",
            "agent/security-policy.toml",
            "tools/security-lane.sh",
        ]);
    }

    paths
}

fn level_required_lanes(level: InitLevel) -> Vec<String> {
    match level {
        InitLevel::Agents => vec![],
        InitLevel::Score => vec!["audit".into(), "doctor".into()],
        InitLevel::Ci => vec!["audit".into(), "doctor".into(), "security".into()],
        InitLevel::Full => unreachable!("full keeps the profile manifest unchanged"),
    }
}

fn level_optional_lanes(level: InitLevel) -> Vec<String> {
    match level {
        InitLevel::Agents => vec![],
        InitLevel::Score => vec![],
        InitLevel::Ci => vec!["ratchet".into()],
        InitLevel::Full => unreachable!("full keeps the profile manifest unchanged"),
    }
}

fn level_validation_commands(level: InitLevel) -> Vec<String> {
    match level {
        InitLevel::Agents => vec!["jankurai adapters verify".into()],
        InitLevel::Score => vec![
            "jankurai doctor --fail-on critical".into(),
            "jankurai audit . --mode advisory --json agent/repo-score.json --md agent/repo-score.md"
                .into(),
        ],
        InitLevel::Ci => vec![
            "jankurai doctor --fail-on critical".into(),
            "jankurai audit . --mode advisory --json target/jankurai/repo-score.json --md target/jankurai/repo-score.md".into(),
            "jankurai ci install . --github --mode ratchet --baseline target/jankurai/baseline-score.json".into(),
        ],
        InitLevel::Full => unreachable!("full keeps the profile manifest unchanged"),
    }
}
