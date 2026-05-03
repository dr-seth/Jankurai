use crate::commands::context_data::RepoCatalog;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct CellManifest {
    pub cell_id: String,
    pub version: String,
    pub category: String,
    pub lifecycle: String,
    pub supported_profiles: Vec<String>,
    pub dependencies: Vec<String>,
    pub source_paths: Vec<String>,
    pub generated_paths: Vec<String>,
    pub contract_paths: Vec<String>,
    pub migration_paths: Vec<String>,
    pub ui_routes: Vec<String>,
    pub proof_lanes: Vec<String>,
    pub proof_commands: Vec<String>,
    pub security_assumptions: Vec<String>,
    pub observability_events: Vec<String>,
    pub docs: Vec<String>,
    pub upgrade_notes: Vec<String>,
    pub rollback_notes: Vec<String>,
    pub certification_status: String,
    pub certification_evidence: Vec<CellEvidence>,
    pub install_strategy: String,
    pub conflict_policy: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CellEvidence {
    pub kind: String,
    pub path: String,
    pub required: bool,
    pub status: String,
}

#[derive(Debug, Clone, Copy)]
pub struct EvidenceCounts {
    pub present: usize,
    pub missing: usize,
    pub review_required: usize,
}

pub fn built_in_manifests(repo: &Path, catalog: &RepoCatalog) -> Vec<CellManifest> {
    vec![
        audit_log_manifest(repo, catalog),
        crud_resource_manifest(repo, catalog),
        rbac_manifest(repo, catalog),
    ]
}

pub fn manifest_for_cell(repo: &Path, catalog: &RepoCatalog, cell_id: &str) -> CellManifest {
    built_in_manifests(repo, catalog)
        .into_iter()
        .find(|manifest| manifest.cell_id == cell_id)
        .unwrap_or_else(|| fallback_manifest(catalog, cell_id))
}

pub fn evidence_counts(manifest: &CellManifest) -> EvidenceCounts {
    let mut counts = EvidenceCounts {
        present: 0,
        missing: 0,
        review_required: 0,
    };
    for evidence in &manifest.certification_evidence {
        match evidence.status.as_str() {
            "present" => counts.present += 1,
            "missing" => counts.missing += 1,
            "review-required" => counts.review_required += 1,
            _ => {}
        }
    }
    counts
}

pub fn owner_for_cell(catalog: &RepoCatalog, cell_id: &str) -> String {
    if let Some(owner) = cell_id.split_once('-').map(|(owner, _)| owner.to_string()) {
        if catalog.owners.values().any(|candidate| candidate == &owner) {
            return owner;
        }
    }
    "workspace".to_string()
}

pub fn category_for_owner(owner: &str) -> &'static str {
    match owner {
        "agent" => "agent-surface",
        "paper" => "documentation",
        "ops" => "governance",
        "standard" => "standard",
        "tools" => "tooling",
        _ => "engineering",
    }
}

pub fn source_paths_for_owner(catalog: &RepoCatalog, owner: &str) -> Vec<String> {
    let mut paths = catalog.prefixes_for_owner(owner);
    if paths.is_empty() {
        paths.push("crates/jankurai/src/commands/".to_string());
    }
    paths
}

fn audit_log_manifest(repo: &Path, catalog: &RepoCatalog) -> CellManifest {
    let source_paths = strings(&[
        "examples/perfect-web-api-db/backend/src/domain.rs",
        "examples/perfect-web-api-db/backend/src/application.rs",
        "examples/perfect-web-api-db/backend/src/adapters.rs",
        "examples/perfect-web-api-db/ops/observability.md",
        "examples/perfect-web-api-db/docs/architecture.md",
        "examples/perfect-web-api-db/README.md",
    ]);
    let contract_paths = strings(&["examples/perfect-web-api-db/contracts/openapi.json"]);
    let migration_paths = strings(&[
        "examples/perfect-web-api-db/db/migrations/001_init.sql",
        "examples/perfect-web-api-db/db/constraints/001_accounts.sql",
    ]);
    let proof_lanes = strings(&["test-cli", "audit", "db-migration-analyze", "security"]);
    certified_manifest(
        repo,
        catalog,
        CellManifest {
            cell_id: "audit-log".to_string(),
            version: "0.1.0".to_string(),
            category: "observability".to_string(),
            lifecycle: "certified".to_string(),
            supported_profiles: strings(&["perfect-web-api-db"]),
            dependencies: Vec::new(),
            source_paths,
            generated_paths: Vec::new(),
            contract_paths,
            migration_paths,
            ui_routes: Vec::new(),
            proof_lanes,
            proof_commands: Vec::new(),
            security_assumptions: strings(&[
                "audit events are append-only and written through the application port",
                "actors and targets are traceable without embedding secrets",
            ]),
            observability_events: strings(&[
                "audit_events_total",
                "resource.created",
                "resource.deleted",
            ]),
            docs: strings(&[
                "examples/perfect-web-api-db/ops/observability.md",
                "examples/perfect-web-api-db/ops/security.md",
                "examples/perfect-web-api-db/docs/architecture.md",
            ]),
            upgrade_notes: strings(&[
                "extend the AuditLog port before adding provider-specific sinks",
                "regenerate client contracts when audit event APIs become public",
            ]),
            rollback_notes: strings(&[
                "dry-run install writes no files",
                "for applied templates, remove audit table additions only through a reviewed migration",
            ]),
            certification_status: "candidate".to_string(),
            certification_evidence: Vec::new(),
            install_strategy: "dry-run-plan".to_string(),
            conflict_policy: "never-overwrite".to_string(),
        },
    )
}

fn crud_resource_manifest(repo: &Path, catalog: &RepoCatalog) -> CellManifest {
    let source_paths = strings(&[
        "examples/perfect-web-api-db/backend/src/domain.rs",
        "examples/perfect-web-api-db/backend/src/application.rs",
        "examples/perfect-web-api-db/backend/src/adapters.rs",
        "examples/perfect-web-api-db/frontend/src/App.tsx",
        "examples/perfect-web-api-db/docs/architecture.md",
        "examples/perfect-web-api-db/README.md",
    ]);
    let contract_paths = strings(&["examples/perfect-web-api-db/contracts/openapi.json"]);
    let migration_paths = strings(&["examples/perfect-web-api-db/db/migrations/001_init.sql"]);
    let ui_routes = strings(&["examples/perfect-web-api-db/ux/routes.md"]);
    let proof_lanes = strings(&[
        "test-cli",
        "audit",
        "db-migration-analyze",
        "ux-qa",
        "security",
    ]);
    certified_manifest(
        repo,
        catalog,
        CellManifest {
            cell_id: "crud-resource".to_string(),
            version: "0.1.0".to_string(),
            category: "product-ui".to_string(),
            lifecycle: "certified".to_string(),
            supported_profiles: strings(&["perfect-web-api-db"]),
            dependencies: strings(&["audit-log"]),
            source_paths,
            generated_paths: Vec::new(),
            contract_paths,
            migration_paths,
            ui_routes,
            proof_lanes,
            proof_commands: Vec::new(),
            security_assumptions: strings(&[
                "CRUD authorization is enforced in the Rust application layer",
                "frontend uses contract-shaped data and does not own durable truth",
            ]),
            observability_events: strings(&["resource.created", "resource.deleted"]),
            docs: strings(&[
                "examples/perfect-web-api-db/ux/routes.md",
                "examples/perfect-web-api-db/ops/security.md",
                "examples/perfect-web-api-db/docs/architecture.md",
            ]),
            upgrade_notes: strings(&[
                "add generated clients before exposing more resource endpoints",
                "expand route states in ux/routes.md with every new CRUD surface",
            ]),
            rollback_notes: strings(&[
                "dry-run install writes no files",
                "for applied templates, reverse resource tables through reviewed migrations",
            ]),
            certification_status: "candidate".to_string(),
            certification_evidence: Vec::new(),
            install_strategy: "dry-run-plan".to_string(),
            conflict_policy: "never-overwrite".to_string(),
        },
    )
}

fn rbac_manifest(repo: &Path, catalog: &RepoCatalog) -> CellManifest {
    let source_paths = strings(&[
        "examples/perfect-web-api-db/backend/src/domain.rs",
        "examples/perfect-web-api-db/backend/src/application.rs",
        "examples/perfect-web-api-db/backend/src/adapters.rs",
        "examples/perfect-web-api-db/docs/architecture.md",
        "examples/perfect-web-api-db/docs/exceptions.md",
        "examples/perfect-web-api-db/README.md",
    ]);
    let contract_paths = strings(&["examples/perfect-web-api-db/contracts/openapi.json"]);
    let migration_paths = strings(&[
        "examples/perfect-web-api-db/db/migrations/001_init.sql",
        "examples/perfect-web-api-db/db/constraints/001_accounts.sql",
    ]);
    let ui_routes = strings(&["examples/perfect-web-api-db/ux/routes.md"]);
    let proof_lanes = strings(&[
        "test-cli",
        "audit",
        "db-migration-analyze",
        "ux-qa",
        "security",
    ]);
    certified_manifest(
        repo,
        catalog,
        CellManifest {
            cell_id: "rbac".to_string(),
            version: "0.1.0".to_string(),
            category: "authorization".to_string(),
            lifecycle: "certified".to_string(),
            supported_profiles: strings(&["perfect-web-api-db"]),
            dependencies: strings(&["crud-resource"]),
            source_paths,
            generated_paths: Vec::new(),
            contract_paths,
            migration_paths,
            ui_routes,
            proof_lanes,
            proof_commands: Vec::new(),
            security_assumptions: strings(&[
                "roles and permissions are enforced in the Rust application layer before any command runs",
                "API security schemes in OpenAPI align with session or token checks at the edge",
                "dangerous role changes emit audit events through the audit-log cell",
            ]),
            observability_events: strings(&["authorization.denied", "authorization.allowed"]),
            docs: strings(&[
                "examples/perfect-web-api-db/ops/security.md",
                "examples/perfect-web-api-db/docs/architecture.md",
                "examples/perfect-web-api-db/docs/exceptions.md",
            ]),
            upgrade_notes: strings(&[
                "model new roles in domain.rs before exposing them in OpenAPI",
                "expand ux/routes.md with permission-denied coverage for each protected surface",
            ]),
            rollback_notes: strings(&[
                "dry-run install writes no files",
                "reverse RBAC table or policy changes only through reviewed migrations",
            ]),
            certification_status: "candidate".to_string(),
            certification_evidence: Vec::new(),
            install_strategy: "dry-run-plan".to_string(),
            conflict_policy: "never-overwrite".to_string(),
        },
    )
}

fn certified_manifest(
    repo: &Path,
    catalog: &RepoCatalog,
    mut manifest: CellManifest,
) -> CellManifest {
    let mut evidence = Vec::new();
    for path in manifest
        .source_paths
        .iter()
        .chain(manifest.contract_paths.iter())
        .chain(manifest.migration_paths.iter())
        .chain(manifest.ui_routes.iter())
        .chain(manifest.docs.iter())
    {
        evidence.push(path_evidence(repo, path));
    }
    for lane in &manifest.proof_lanes {
        evidence.push(lane_evidence(catalog, lane));
    }
    manifest.proof_commands = proof_commands(catalog, &manifest.proof_lanes);
    manifest.certification_status = if evidence
        .iter()
        .all(|item| !item.required || item.status == "present")
    {
        "certified".to_string()
    } else {
        "candidate".to_string()
    };
    manifest.certification_evidence = evidence;
    manifest
}

fn fallback_manifest(catalog: &RepoCatalog, cell_id: &str) -> CellManifest {
    let owner = owner_for_cell(catalog, cell_id);
    let source_paths = source_paths_for_owner(catalog, &owner);
    let proof_lanes = if catalog.proof_lane_names().is_empty() {
        strings(&["fast", "audit"])
    } else {
        catalog.proof_lane_names()
    };
    CellManifest {
        cell_id: cell_id.to_string(),
        version: "0.0.0".to_string(),
        category: category_for_owner(&owner).to_string(),
        lifecycle: "draft".to_string(),
        supported_profiles: strings(&["workspace"]),
        dependencies: Vec::new(),
        source_paths,
        generated_paths: Vec::new(),
        contract_paths: Vec::new(),
        migration_paths: Vec::new(),
        ui_routes: Vec::new(),
        proof_lanes: proof_lanes.clone(),
        proof_commands: proof_commands(catalog, &proof_lanes),
        security_assumptions: Vec::new(),
        observability_events: Vec::new(),
        docs: Vec::new(),
        upgrade_notes: strings(&[
            "add local contracts, tests, and proof lanes before certification",
        ]),
        rollback_notes: strings(&["dry-run install writes no files"]),
        certification_status: "candidate".to_string(),
        certification_evidence: vec![CellEvidence {
            kind: "review".to_string(),
            path: cell_id.to_string(),
            required: true,
            status: "review-required".to_string(),
        }],
        install_strategy: "manual".to_string(),
        conflict_policy: "review-required".to_string(),
    }
}

fn path_evidence(repo: &Path, path: &str) -> CellEvidence {
    CellEvidence {
        kind: "path".to_string(),
        path: path.to_string(),
        required: true,
        status: if repo.join(path).exists() {
            "present".to_string()
        } else {
            "missing".to_string()
        },
    }
}

fn lane_evidence(catalog: &RepoCatalog, lane: &str) -> CellEvidence {
    CellEvidence {
        kind: "proof-lane".to_string(),
        path: lane.to_string(),
        required: true,
        status: if catalog.proof_lanes.iter().any(|item| item.name == lane) {
            "present".to_string()
        } else {
            "missing".to_string()
        },
    }
}

fn proof_commands(catalog: &RepoCatalog, lanes: &[String]) -> Vec<String> {
    let mut commands = Vec::new();
    for lane_name in lanes {
        for lane in &catalog.proof_lanes {
            if lane.name == *lane_name && !commands.contains(&lane.command) {
                commands.push(lane.command.clone());
            }
        }
    }
    commands
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}
