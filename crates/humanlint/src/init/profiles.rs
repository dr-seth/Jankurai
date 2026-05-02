use crate::validation::{self, ArtifactSchema};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

const RUST_TS_POSTGRES_JSON: &str = include_str!("../../templates/profiles/rust-ts-postgres.json");
const RUST_API_JSON: &str = include_str!("../../templates/profiles/rust-api.json");
const REACT_WEB_JSON: &str = include_str!("../../templates/profiles/react-web.json");
const B2B_SAAS_JSON: &str = include_str!("../../templates/profiles/b2b-saas.json");

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

fn load_profile(repo: &Path, json_str: &str) -> Result<ProfileManifest> {
    let value: serde_json::Value = serde_json::from_str(json_str)?;
    validation::validate_value(repo, ArtifactSchema::InitProfile, &value)?;
    let file: ProfileManifestFile = serde_json::from_value(value)?;
    Ok(file.into())
}

/// Resolves a bundled init profile.
pub fn resolve_profile(repo: &Path, profile: &str) -> Result<ProfileManifest> {
    match normalize_profile_id(profile) {
        "rust-ts-postgres" => load_rust_ts_postgres(repo),
        "rust-api" => load_rust_api(repo),
        "react-web" => load_react_web(repo),
        "b2b-saas" => load_b2b_saas(repo),
        other => bail!(
            "unknown init profile `{}`. supported: rust-ts-postgres, rust-api, react-web, b2b-saas (plus aliases)",
            other
        ),
    }
}

fn normalize_profile_id(profile: &str) -> &str {
    match profile {
        "rust-ts-postgres"
        | "rust-ts-vite-react-postgres"
        | "rust-ts-vite-react-postgres-bounded-python" => "rust-ts-postgres",
        other => other,
    }
}
