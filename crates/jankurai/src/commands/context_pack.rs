use crate::commands::context_data::{push_unique, RepoCatalog};
use crate::validation::{self, ArtifactSchema};
use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ContextPackArgs {
    pub repo: PathBuf,
    pub task: String,
    pub changed: Vec<PathBuf>,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextPack {
    pub schema_version: String,
    pub task: String,
    pub owner: String,
    pub permission_profile: String,
    pub changed_paths: Vec<String>,
    pub allowed_paths: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub generated_zones: Vec<String>,
    pub read_first_files: Vec<String>,
    pub relevant_docs: Vec<String>,
    pub proof_lanes: Vec<String>,
    pub commands: Vec<String>,
    pub likely_rules: Vec<String>,
    pub max_context_files: usize,
    pub stop_conditions: Vec<String>,
    pub residual_risk: Vec<String>,
    pub token_budget: usize,
}

pub fn run(args: ContextPackArgs) -> Result<()> {
    let pack = build_context_pack(&args.repo, &args.task, &args.changed)?;
    match args.out.as_deref() {
        Some(path) => {
            validation::write_json(&args.repo, ArtifactSchema::ContextPack, path, &pack)?;
        }
        None => {
            validation::validate_serializable(&args.repo, ArtifactSchema::ContextPack, &pack)?;
            println!("{}", serde_json::to_string_pretty(&pack)?);
        }
    }
    if let Some(path) = args.md.as_deref() {
        crate::render::write_markdown(path, &render_markdown(&pack))?;
    }
    Ok(())
}

pub fn build_context_pack(repo: &Path, task: &str, changed: &[PathBuf]) -> Result<ContextPack> {
    let catalog = RepoCatalog::load(repo)?;
    let changed_paths = normalize_paths(changed);
    let task_lc = task.to_ascii_lowercase();
    let mut allowed_paths = Vec::new();

    if !changed_paths.is_empty() {
        for path in &changed_paths {
            push_unique(&mut allowed_paths, path.clone());
            for prefix in prefixes_for_path(&catalog, path) {
                push_unique(&mut allowed_paths, prefix);
            }
        }
    }

    if allowed_paths.is_empty() {
        for path in infer_roots(&task_lc) {
            push_unique(&mut allowed_paths, path);
        }
    }

    let owner = infer_owner(&catalog, &task_lc, &changed_paths, &allowed_paths);
    if owner == "mixed" {
        for prefix in catalog.prefixes_for_owner("agent") {
            push_unique(&mut allowed_paths, prefix);
        }
    } else if owner != "unmapped" {
        for prefix in catalog.prefixes_for_owner(&owner) {
            push_unique(&mut allowed_paths, prefix);
        }
    }

    let permission_profile = infer_permission_profile(&task_lc, &allowed_paths, &owner);
    let generated_zones = catalog.generated_paths();
    let forbidden_paths = build_forbidden_paths(&catalog);
    let read_first_files = build_read_first_files(&task_lc);
    let relevant_docs = build_relevant_docs(&task_lc);
    let proof_lanes = build_proof_lanes(&task_lc, &allowed_paths, &permission_profile);
    let mut commands = catalog.commands_for_paths(&allowed_paths);
    for lane in &proof_lanes {
        let lane_cmds = catalog.proof_lane_commands(&[lane.as_str()]);
        for cmd in lane_cmds {
            push_unique(&mut commands, cmd);
        }
    }
    if commands.is_empty() {
        for cmd in fallback_commands(&permission_profile) {
            push_unique(&mut commands, cmd);
        }
    }
    let likely_rules = build_likely_rules(&task_lc, &permission_profile, &owner, &allowed_paths);
    let stop_conditions =
        build_stop_conditions(&permission_profile, &allowed_paths, &generated_zones);
    let residual_risk = vec![
        "heuristic routing can miss a cross-owner edit".to_string(),
        "confirm the source contract before editing generated output".to_string(),
    ];
    Ok(ContextPack {
        schema_version: "1.0.0".to_string(),
        task: task.to_string(),
        owner,
        permission_profile,
        changed_paths,
        allowed_paths,
        forbidden_paths,
        generated_zones,
        read_first_files,
        relevant_docs,
        proof_lanes,
        commands,
        likely_rules,
        max_context_files: 12,
        stop_conditions,
        residual_risk,
        token_budget: 100000,
    })
}

fn render_markdown(pack: &ContextPack) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Context Pack");
    let _ = writeln!(out);
    let _ = writeln!(out, "- task: `{}`", pack.task);
    let _ = writeln!(out, "- owner: `{}`", pack.owner);
    let _ = writeln!(out, "- permission profile: `{}`", pack.permission_profile);
    let _ = writeln!(out, "- changed: `{}`", join_or_none(&pack.changed_paths));
    let _ = writeln!(out, "- allowed: `{}`", join_or_none(&pack.allowed_paths));
    let _ = writeln!(
        out,
        "- forbidden: `{}`",
        join_or_none(&pack.forbidden_paths)
    );
    let _ = writeln!(
        out,
        "- generated: `{}`",
        join_or_none(&pack.generated_zones)
    );
    let _ = writeln!(
        out,
        "- read first: `{}`",
        join_or_none(&pack.read_first_files)
    );
    let _ = writeln!(out, "- docs: `{}`", join_or_none(&pack.relevant_docs));
    let _ = writeln!(out, "- proof lanes: `{}`", join_or_none(&pack.proof_lanes));
    let _ = writeln!(out, "- commands: `{}`", join_or_none(&pack.commands));
    let _ = writeln!(
        out,
        "- likely rules: `{}`",
        join_or_none(&pack.likely_rules)
    );
    let _ = writeln!(out, "- max context files: `{}`", pack.max_context_files);
    let _ = writeln!(out, "- stop: `{}`", join_or_none(&pack.stop_conditions));
    let _ = writeln!(
        out,
        "- residual risk: `{}`",
        join_or_none(&pack.residual_risk)
    );
    let _ = writeln!(out, "- token budget: `{}`", pack.token_budget);
    out
}

fn infer_owner(
    catalog: &RepoCatalog,
    task_lc: &str,
    changed_paths: &[String],
    allowed_paths: &[String],
) -> String {
    let mut owners = Vec::new();
    for path in changed_paths.iter().chain(allowed_paths.iter()) {
        if let Some(owner) = catalog.owner_for_path(path) {
            push_unique(&mut owners, owner.to_string());
        }
    }
    if owners.len() == 1 {
        return owners[0].clone();
    }
    if owners.len() > 1 {
        return "mixed".to_string();
    }
    if task_lc.contains("agent")
        || task_lc.contains("context")
        || task_lc.contains("repair")
        || task_lc.contains("adapter")
    {
        "agent".to_string()
    } else if task_lc.contains("ux")
        || task_lc.contains("web")
        || task_lc.contains("storybook")
        || task_lc.contains("playwright")
    {
        "tools".to_string()
    } else if task_lc.contains("security") || task_lc.contains("secret") {
        "ops".to_string()
    } else if task_lc.contains("paper") || task_lc.contains("moonshot") {
        "paper".to_string()
    } else if task_lc.contains("docs") {
        "standard".to_string()
    } else if task_lc.is_empty() {
        "unmapped".to_string()
    } else {
        "mixed".to_string()
    }
}

fn infer_permission_profile(task_lc: &str, allowed_paths: &[String], owner: &str) -> String {
    if task_lc.contains("security") || owner == "ops" {
        "security-investigation".to_string()
    } else if task_lc.contains("paper") || allowed_paths.iter().any(|p| p.starts_with("paper/")) {
        "docs-only".to_string()
    } else if task_lc.contains("docs")
        || allowed_paths
            .iter()
            .all(|path| path.starts_with("docs/") || path == "AGENTS.md")
    {
        "docs-only".to_string()
    } else if task_lc.contains("generated") || allowed_paths.iter().any(|p| p.contains(".toml")) {
        "generated-regeneration".to_string()
    } else if task_lc.contains("release") {
        "release".to_string()
    } else if task_lc.is_empty() {
        "read-only".to_string()
    } else {
        "code-edit".to_string()
    }
}

fn build_forbidden_paths(catalog: &RepoCatalog) -> Vec<String> {
    let mut out = vec!["reference/".to_string(), "target/".to_string()];
    for path in catalog.forbidden_generated_paths() {
        push_unique(&mut out, path);
    }
    out
}

fn build_read_first_files(task_lc: &str) -> Vec<String> {
    let mut out = vec![
        "AGENTS.md".to_string(),
        "agent/JANKURAI_STANDARD.md".to_string(),
        "docs/agent-native-standard.md".to_string(),
        "docs/moonshot.md".to_string(),
    ];
    if task_lc.contains("agent") || task_lc.contains("repair") || task_lc.contains("context") {
        push_unique(&mut out, "docs/boundary-oracle.md");
        push_unique(&mut out, "docs/install.md");
        push_unique(&mut out, "docs/ide-integrations.md");
        push_unique(&mut out, "tips/phases/08-agent-context-repair.md");
    }
    if task_lc.contains("reference") || task_lc.contains("platform") || task_lc.contains("golden") {
        push_unique(&mut out, "tips/phases/09-reference-product-platform.md");
        push_unique(&mut out, "docs/streaming.md");
    }
    if task_lc.contains("security") {
        push_unique(&mut out, "docs/boundary-oracle.md");
    }
    out
}

fn build_relevant_docs(task_lc: &str) -> Vec<String> {
    let mut out = vec![
        "docs/agent-native-standard.md".to_string(),
        "docs/moonshot.md".to_string(),
    ];
    if task_lc.contains("agent") || task_lc.contains("repair") || task_lc.contains("context") {
        push_unique(&mut out, "docs/boundary-oracle.md");
        push_unique(&mut out, "docs/install.md");
        push_unique(&mut out, "docs/ide-integrations.md");
    }
    if task_lc.contains("reference") || task_lc.contains("platform") || task_lc.contains("golden") {
        push_unique(&mut out, "docs/streaming.md");
        push_unique(&mut out, "tips/phases/09-reference-product-platform.md");
    }
    if task_lc.contains("security") {
        push_unique(&mut out, "docs/boundary-oracle.md");
    }
    if task_lc.contains("docs") {
        push_unique(&mut out, "docs/ide-integrations.md");
    }
    out
}

fn build_proof_lanes(
    task_lc: &str,
    allowed_paths: &[String],
    permission_profile: &str,
) -> Vec<String> {
    let mut lanes = vec!["fast".to_string(), "audit".to_string()];
    if permission_profile == "security-investigation" || task_lc.contains("security") {
        push_unique(&mut lanes, "security");
    }
    if task_lc.contains("paper") || allowed_paths.iter().any(|p| p.starts_with("paper/")) {
        push_unique(&mut lanes, "paper");
    }
    if task_lc.contains("reference")
        || task_lc.contains("platform")
        || task_lc.contains("golden")
        || task_lc.contains("ux")
        || task_lc.contains("web")
        || task_lc.contains("db")
        || task_lc.contains("contract")
    {
        push_unique(&mut lanes, "full");
    }
    if task_lc.contains("release") {
        push_unique(&mut lanes, "full");
    }
    lanes
}

fn build_likely_rules(
    task_lc: &str,
    permission_profile: &str,
    owner: &str,
    allowed_paths: &[String],
) -> Vec<String> {
    let mut out = Vec::new();
    if permission_profile == "security-investigation" || task_lc.contains("security") {
        out.extend([
            "HLT-010-SECRET-SPRAWL".to_string(),
            "HLT-011-PROMPT-INJECTION".to_string(),
            "HLT-012-OVERBROAD-AGENCY".to_string(),
        ]);
    }
    if task_lc.contains("context") || task_lc.contains("repair") || owner == "agent" {
        out.extend([
            "HLT-011-PROMPT-INJECTION".to_string(),
            "HLT-012-OVERBROAD-AGENCY".to_string(),
            "HLT-015-CONTEXT-SETUP-GAP".to_string(),
            "HLT-017-OPAQUE-OBSERVABILITY".to_string(),
        ]);
    }
    if task_lc.contains("reference")
        || task_lc.contains("platform")
        || task_lc.contains("golden")
        || allowed_paths
            .iter()
            .any(|p| p.starts_with("packages/ux-qa/"))
    {
        out.extend([
            "HLT-004-UNMAPPED-PROOF".to_string(),
            "HLT-007-HANDWRITTEN-CONTRACT".to_string(),
            "HLT-013-RENDERED-UX-GAP".to_string(),
        ]);
    }
    if task_lc.contains("db") || task_lc.contains("schema") || task_lc.contains("contract") {
        out.extend([
            "HLT-006-DIRECT-DB-WRONG-LAYER".to_string(),
            "HLT-007-HANDWRITTEN-CONTRACT".to_string(),
        ]);
    }
    if task_lc.contains("generated") {
        out.push("HLT-002-GENERATED-MUTATION".to_string());
    }
    if task_lc.contains("streaming") {
        out.push("HLT-019-STREAMING-RUNTIME-DRIFT".to_string());
    }
    if out.is_empty() {
        out.extend([
            "HLT-003-OWNERLESS-PATH".to_string(),
            "HLT-004-UNMAPPED-PROOF".to_string(),
        ]);
    }
    out.sort();
    out.dedup();
    out
}

fn build_stop_conditions(
    permission_profile: &str,
    allowed_paths: &[String],
    generated_zones: &[String],
) -> Vec<String> {
    let mut out = vec![
        "stop if the requested edit would touch `reference/`".to_string(),
        "stop if the requested edit would broaden the permission profile".to_string(),
        "stop if the change requires a new generated artifact without a source contract"
            .to_string(),
    ];
    if permission_profile == "security-investigation" {
        push_unique(
            &mut out,
            "stop and escalate for secrets, credentials, or token material",
        );
    }
    if allowed_paths.iter().any(|path| path.starts_with("docs/")) {
        push_unique(
            &mut out,
            "stop if the task would introduce product-runtime truth into docs",
        );
    }
    if !generated_zones.is_empty() {
        push_unique(
            &mut out,
            "stop if any edit lands in a generated zone instead of the declared source",
        );
    }
    out
}

fn fallback_commands(permission_profile: &str) -> Vec<String> {
    let mut out = vec!["just fast".to_string(), "just score".to_string()];
    if permission_profile == "security-investigation" {
        push_unique(&mut out, "just security");
    }
    out
}

fn prefixes_for_path(catalog: &RepoCatalog, path: &str) -> Vec<String> {
    let mut out = Vec::new();
    for prefix in catalog.owners.keys() {
        if path_matches(path, prefix) {
            push_unique(&mut out, prefix.clone());
        }
    }
    out
}

fn infer_roots(task_lc: &str) -> Vec<String> {
    let mut out = Vec::new();
    if task_lc.contains("agent") || task_lc.contains("context") || task_lc.contains("repair") {
        out.extend([
            "agent/".to_string(),
            "crates/jankurai/src/commands/".to_string(),
            "docs/".to_string(),
            "tips/phases/08-agent-context-repair.md".to_string(),
        ]);
    }
    if task_lc.contains("reference") || task_lc.contains("platform") || task_lc.contains("golden") {
        out.extend([
            "docs/moonshot.md".to_string(),
            "tips/phases/09-reference-product-platform.md".to_string(),
        ]);
    }
    if task_lc.contains("ux") || task_lc.contains("web") {
        out.push("packages/ux-qa/".to_string());
    }
    if task_lc.contains("db") || task_lc.contains("schema") || task_lc.contains("contract") {
        out.extend(["db/".to_string(), "schemas/".to_string()]);
    }
    if task_lc.contains("security") {
        out.extend([
            ".github/".to_string(),
            "docs/boundary-oracle.md".to_string(),
        ]);
    }
    if task_lc.contains("paper") {
        out.push("tips/".to_string());
    }
    if out.is_empty() && !task_lc.is_empty() {
        out.extend(["agent/".to_string(), "docs/".to_string()]);
    }
    out
}

fn normalize_paths(paths: &[PathBuf]) -> Vec<String> {
    let mut out = Vec::new();
    for path in paths {
        let text = path.to_string_lossy().replace('\\', "/");
        push_unique(&mut out, text);
    }
    out
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn path_matches(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(prefix) || prefix.starts_with(path)
}
