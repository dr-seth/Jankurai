use anyhow::{bail, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct InitPlan {
    pub profile: String,
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
    ide: &str,
    mode: &str,
    ci: &str,
    issue_backend: &str,
    ux_qa: bool,
) -> Result<InitPlan> {
    let existing = super::detect::existing_standard_files(repo);
    let detected = super::detect::detect_surfaces(repo);
    let profile_manifest = match profile_file {
        Some(path) => super::profiles::load_profile_from_path(repo, path)?,
        None => super::profiles::resolve_profile(repo, profile)?,
    };
    let profile = profile_manifest.id.clone();
    let mut paths = profile_manifest.generated_paths.clone();
    paths.sort();
    let mut actions = Vec::new();
    for path in &paths {
        if super::templates::template_for_path(path).is_none() {
            bail!("profile declares `{path}` but no init template is registered (see init/templates.rs)");
        }
        let action = if repo.join(path).exists() {
            profile_manifest.merge_policy_for_path(path).plan_action().into()
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
