use crate::validation::{self, ArtifactSchema};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

const RUST_TS_POSTGRES_JSON: &str = include_str!("../../templates/profiles/rust-ts-postgres.json");
const RUST_API_JSON: &str = include_str!("../../templates/profiles/rust-api.json");
const REACT_WEB_JSON: &str = include_str!("../../templates/profiles/react-web.json");
const B2B_SAAS_JSON: &str = include_str!("../../templates/profiles/b2b-saas.json");
const AI_PRODUCT_JSON: &str = include_str!("../../templates/profiles/ai-product.json");
const REGULATED_SAAS_JSON: &str = include_str!("../../templates/profiles/regulated-saas.json");
const MIGRATION_TARGET_JSON: &str = include_str!("../../templates/profiles/migration-target.json");

/// Bundled profile IDs accepted by `resolve_profile` after alias normalization.
pub const BUNDLED_PROFILE_IDS: &[&str] = &[
    "rust-ts-postgres",
    "rust-api",
    "react-web",
    "b2b-saas",
    "ai-product",
    "regulated-saas",
    "migration-target",
];

#[derive(Debug, Clone, Serialize)]
pub struct ProfileManifest {
    pub id: String,
    pub display_name: String,
    pub target_stack_id: String,
    pub generated_paths: Vec<String>,
    pub required_lanes: Vec<String>,
    pub optional_lanes: Vec<String>,
    pub agent_adapters: Vec<String>,
    pub ci_templates: Vec<String>,
    pub docs: Vec<String>,
    pub security_controls: Vec<String>,
    pub ux_controls: Vec<String>,
    pub contract_system: Vec<String>,
    pub db_policy: Vec<String>,
    pub validation_commands: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileManifestFile {
    id: String,
    display_name: String,
    target_stack_id: String,
    generated_paths: Vec<String>,
    required_lanes: Vec<String>,
    optional_lanes: Vec<String>,
    agent_adapters: Vec<String>,
    ci_templates: Vec<String>,
    docs: Vec<String>,
    security_controls: Vec<String>,
    ux_controls: Vec<String>,
    contract_system: Vec<String>,
    db_policy: Vec<String>,
    validation_commands: Vec<String>,
}

impl From<ProfileManifestFile> for ProfileManifest {
    fn from(f: ProfileManifestFile) -> Self {
        Self {
            id: f.id,
            display_name: f.display_name,
            target_stack_id: f.target_stack_id,
            generated_paths: f.generated_paths,
            required_lanes: f.required_lanes,
            optional_lanes: f.optional_lanes,
            agent_adapters: f.agent_adapters,
            ci_templates: f.ci_templates,
            docs: f.docs,
            security_controls: f.security_controls,
            ux_controls: f.ux_controls,
            contract_system: f.contract_system,
            db_policy: f.db_policy,
            validation_commands: f.validation_commands,
        }
    }
}

fn load_rust_ts_postgres(repo: &Path) -> Result<ProfileManifest> {
    load_profile(repo, RUST_TS_POSTGRES_JSON)
}

fn load_rust_api(repo: &Path) -> Result<ProfileManifest> {
    load_profile(repo, RUST_API_JSON)
}

fn load_react_web(repo: &Path) -> Result<ProfileManifest> {
    load_profile(repo, REACT_WEB_JSON)
}

fn load_b2b_saas(repo: &Path) -> Result<ProfileManifest> {
    load_profile(repo, B2B_SAAS_JSON)
}

fn load_ai_product(repo: &Path) -> Result<ProfileManifest> {
    load_profile(repo, AI_PRODUCT_JSON)
}

fn load_regulated_saas(repo: &Path) -> Result<ProfileManifest> {
    load_profile(repo, REGULATED_SAAS_JSON)
}

fn load_migration_target(repo: &Path) -> Result<ProfileManifest> {
    load_profile(repo, MIGRATION_TARGET_JSON)
}

fn load_profile(repo: &Path, json_str: &str) -> Result<ProfileManifest> {
    let value: serde_json::Value = serde_json::from_str(json_str)?;
    validation::validate_value(repo, ArtifactSchema::InitProfile, &value)?;
    let file: ProfileManifestFile = serde_json::from_value(value)?;
    Ok(file.into())
}

/// Load and validate an init profile manifest from a JSON file (same schema as bundled profiles).
pub fn load_profile_from_path(repo: &Path, path: &Path) -> Result<ProfileManifest> {
    let json_str = std::fs::read_to_string(path)
        .with_context(|| format!("read init profile `{}`", path.display()))?;
    load_profile(repo, &json_str)
}

/// Resolves a bundled init profile.
pub fn resolve_profile(repo: &Path, profile: &str) -> Result<ProfileManifest> {
    match normalize_profile_id(profile) {
        "rust-ts-postgres" => load_rust_ts_postgres(repo),
        "rust-api" => load_rust_api(repo),
        "react-web" => load_react_web(repo),
        "b2b-saas" => load_b2b_saas(repo),
        "ai-product" => load_ai_product(repo),
        "regulated-saas" => load_regulated_saas(repo),
        "migration-target" => load_migration_target(repo),
        other => bail!(
            "unknown init profile `{}`. supported bundled profiles: {}",
            other,
            BUNDLED_PROFILE_IDS.join(", ")
        ),
    }
}

fn normalize_profile_id(profile: &str) -> &str {
    match profile {
        "rust-ts-postgres"
        | "rust-ts-vite-react-postgres"
        | "rust-ts-vite-react-postgres-bounded-python" => "rust-ts-postgres",
        "ai" | "ai-python" | "eval-product" => "ai-product",
        "regulated" | "compliance-saas" | "soc-saas" => "regulated-saas",
        "migration" | "legacy-migration" | "migration-repo" => "migration-target",
        other => other,
    }
}

#[cfg(test)]
mod bundled_profile_contract {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn template_json_files_match_bundled_profile_id_list() {
        let profiles_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates/profiles");
        let mut stems: Vec<String> = fs::read_dir(&profiles_dir)
            .unwrap()
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                if p.extension()?.to_str()? == "json" {
                    Some(p.file_stem()?.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
            .collect();
        stems.sort();
        let mut declared: Vec<String> = BUNDLED_PROFILE_IDS.iter().map(|s| (*s).to_string()).collect();
        declared.sort();
        assert_eq!(
            stems, declared,
            "templates/profiles/*.json stems must exactly match BUNDLED_PROFILE_IDS (no orphan files, no missing profiles)"
        );
    }

    #[test]
    fn each_bundled_manifest_id_matches_its_bundle_key() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..");
        for id in BUNDLED_PROFILE_IDS {
            let manifest = resolve_profile(&repo, id).unwrap();
            assert_eq!(
                manifest.id.as_str(),
                *id,
                "profile `{id}` JSON `id` field must match bundled key (tip: keep filename stem and id aligned)"
            );
        }
    }
}
