use crate::validation::{self, ArtifactSchema};
use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct MigrateArgs {
    pub repo: PathBuf,
    pub out: Option<String>,
    pub md: Option<String>,
    pub mode: MigrateMode,
    pub target: String,
}

#[derive(Debug, Clone)]
pub enum MigrateMode {
    Analyze,
    Plan,
}

// ---------------------------------------------------------------------------
// Structured Inventory (Phase 11 hardening)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct DetectedItem {
    pub name: String,
    pub evidence: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiSurface {
    pub framework: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContractEvidence {
    pub kind: String,
    pub path: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StackInventory {
    pub languages: Vec<DetectedItem>,
    pub frameworks: Vec<DetectedItem>,
    pub db_clients: Vec<DetectedItem>,
    pub test_frameworks: Vec<DetectedItem>,
    pub package_managers: Vec<DetectedItem>,
    pub ci_systems: Vec<DetectedItem>,
    pub api_surfaces: Vec<ApiSurface>,
    pub contract_evidence: Vec<ContractEvidence>,
}

// ---------------------------------------------------------------------------
// Dimensional Liability (Phase 11 hardening)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct LiabilityBreakdown {
    pub total: u32,
    pub dimensions: Vec<LiabilityDimension>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiabilityDimension {
    pub name: String,
    pub score: u32,
    pub weight: f64,
    pub evidence: Vec<String>,
}

// ---------------------------------------------------------------------------
// MigrationReport — matches schemas/migration-report.schema.json
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct MigrationReport {
    pub schema_version: String,
    pub command: String,
    pub status: String,
    pub generated_at: String,
    pub source_root: String,
    pub source_stack: String,
    pub target_stack: String,
    pub liability_score: u32,
    pub liability_breakdown: LiabilityBreakdown,
    pub inventory: StackInventory,
    pub module_inventory: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_guesses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_boundaries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_surfaces: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_surfaces: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_logic: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high_risk_areas: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_tests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strangler_candidates: Option<Vec<String>>,
    pub contract_evidence: Vec<ContractEvidence>,
    pub recommended_slice_order: Vec<String>,
    pub required_proof_lanes: Vec<String>,
    pub rollback_cutover_notes: Vec<String>,
}

// ---------------------------------------------------------------------------
// MigrationPlan — matches schemas/migration-plan.schema.json
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct MigrationPlan {
    pub schema_version: String,
    pub command: String,
    pub status: String,
    pub generated_at: String,
    pub source_report: String,
    pub target_stack: String,
    pub plan_mode: String,
    pub slices: Vec<MigrationSlice>,
    pub human_approval_requirements: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationSlice {
    pub slice_id: String,
    pub owner: String,
    pub status: String,
    pub risk_level: String,
    pub dependency_order: u32,
    pub human_approval_required: bool,
    pub allowed_paths: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub contracts: Vec<String>,
    pub tests: Vec<String>,
    pub proof_lanes: Vec<String>,
    pub rollback_notes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cutover_notes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

pub fn detect_stack(repo: &Path) -> StackInventory {
    let mut inv = StackInventory {
        languages: vec![],
        frameworks: vec![],
        db_clients: vec![],
        test_frameworks: vec![],
        package_managers: vec![],
        ci_systems: vec![],
        api_surfaces: vec![],
        contract_evidence: vec![],
    };

    if repo.join("Cargo.toml").exists() {
        inv.languages.push(di("rust", "Cargo.toml", "high"));
        inv.package_managers.push(di("cargo", "Cargo.toml", "high"));
        inv.test_frameworks
            .push(di("cargo-test", "Cargo.toml", "high"));
        if let Ok(text) = fs::read_to_string(repo.join("Cargo.toml")) {
            let lower = text.to_ascii_lowercase();
            for fw in ["actix", "axum", "rocket", "warp"] {
                if lower.contains(fw) {
                    inv.frameworks.push(di(fw, "Cargo.toml", "medium"));
                    inv.api_surfaces.push(ApiSurface {
                        framework: fw.to_string(),
                        evidence: "Cargo.toml dependency".to_string(),
                    });
                }
            }
            for db in ["sqlx", "diesel", "sea-orm", "tokio-postgres"] {
                if lower.contains(db) {
                    inv.db_clients.push(di(db, "Cargo.toml", "medium"));
                }
            }
        }
    }

    if repo.join("package.json").exists() {
        inv.languages
            .push(di("typescript", "package.json", "medium"));
        inv.package_managers.push(di("npm", "package.json", "high"));
        if let Ok(text) = fs::read_to_string(repo.join("package.json")) {
            let lower = text.to_ascii_lowercase();
            for fw in [
                "express", "fastify", "next", "nuxt", "react", "vue", "angular", "svelte",
            ] {
                if lower.contains(fw) {
                    inv.frameworks.push(di(fw, "package.json", "medium"));
                    if matches!(fw, "express" | "fastify" | "next" | "nuxt") {
                        inv.api_surfaces.push(ApiSurface {
                            framework: fw.to_string(),
                            evidence: "package.json dependency".to_string(),
                        });
                    }
                }
            }
            for tf in ["jest", "vitest", "mocha", "playwright", "cypress"] {
                if lower.contains(tf) {
                    inv.test_frameworks.push(di(tf, "package.json", "medium"));
                }
            }
            for db in ["prisma", "knex", "typeorm", "sequelize", "drizzle"] {
                if lower.contains(db) {
                    inv.db_clients.push(di(db, "package.json", "medium"));
                }
            }
        }
    }

    if repo.join("requirements.txt").exists() || repo.join("pyproject.toml").exists() {
        let evidence = if repo.join("requirements.txt").exists() {
            "requirements.txt"
        } else {
            "pyproject.toml"
        };
        inv.languages.push(di("python", evidence, "high"));
        inv.package_managers.push(di("pip", evidence, "high"));
        inv.test_frameworks.push(di("pytest", evidence, "medium"));
        for manifest in ["requirements.txt", "pyproject.toml"] {
            if let Ok(text) = fs::read_to_string(repo.join(manifest)) {
                let lower = text.to_ascii_lowercase();
                for fw in ["fastapi", "django", "flask"] {
                    if lower.contains(fw) {
                        inv.frameworks.push(di(fw, manifest, "medium"));
                        inv.api_surfaces.push(ApiSurface {
                            framework: fw.to_string(),
                            evidence: format!("{manifest} dependency"),
                        });
                    }
                }
                for db in ["psycopg", "sqlalchemy", "asyncpg"] {
                    if lower.contains(db) {
                        inv.db_clients.push(di(db, manifest, "medium"));
                    }
                }
            }
        }
    }

    if repo.join("pom.xml").exists() || repo.join("build.gradle").exists() {
        let evidence = if repo.join("pom.xml").exists() {
            "pom.xml"
        } else {
            "build.gradle"
        };
        inv.languages.push(di("java", evidence, "high"));
        if repo.join("pom.xml").exists() {
            inv.package_managers.push(di("maven", "pom.xml", "high"));
        }
        if repo.join("build.gradle").exists() {
            inv.package_managers
                .push(di("gradle", "build.gradle", "high"));
        }
        inv.test_frameworks.push(di("junit", evidence, "medium"));
        if let Ok(text) = fs::read_to_string(repo.join("pom.xml")) {
            if text.to_ascii_lowercase().contains("spring") {
                inv.frameworks.push(di("spring", "pom.xml", "medium"));
                inv.api_surfaces.push(ApiSurface {
                    framework: "spring".to_string(),
                    evidence: "pom.xml dependency".to_string(),
                });
            }
        }
    }

    if repo.join("Gemfile").exists() {
        inv.languages.push(di("ruby", "Gemfile", "high"));
        inv.package_managers.push(di("bundler", "Gemfile", "high"));
        inv.test_frameworks.push(di("rspec", "Gemfile", "medium"));
        inv.frameworks.push(di("rails", "Gemfile", "medium"));
        inv.api_surfaces.push(ApiSurface {
            framework: "rails".to_string(),
            evidence: "Gemfile".to_string(),
        });
    }

    if repo.join("composer.json").exists() {
        inv.languages.push(di("php", "composer.json", "high"));
        inv.package_managers
            .push(di("composer", "composer.json", "high"));
        if let Ok(text) = fs::read_to_string(repo.join("composer.json")) {
            if text.to_ascii_lowercase().contains("laravel") {
                inv.frameworks
                    .push(di("laravel", "composer.json", "medium"));
                inv.api_surfaces.push(ApiSurface {
                    framework: "laravel".to_string(),
                    evidence: "composer.json dependency".to_string(),
                });
            }
        }
    }

    if repo.join("go.mod").exists() {
        inv.languages.push(di("go", "go.mod", "high"));
        inv.package_managers
            .push(di("go-modules", "go.mod", "high"));
        inv.test_frameworks.push(di("go-test", "go.mod", "high"));
    }

    // CI detection
    if repo.join(".github/workflows").exists() {
        inv.ci_systems
            .push(di("github-actions", ".github/workflows/", "high"));
    }
    if repo.join(".gitlab-ci.yml").exists() {
        inv.ci_systems
            .push(di("gitlab-ci", ".gitlab-ci.yml", "high"));
    }
    if repo.join(".circleci").exists() {
        inv.ci_systems.push(di("circleci", ".circleci/", "high"));
    }

    // Contract evidence detection
    for (kind, glob_pattern) in [
        ("openapi", "openapi.yaml"),
        ("openapi", "openapi.json"),
        ("openapi", "swagger.json"),
        ("proto", "*.proto"),
        ("graphql", "schema.graphql"),
    ] {
        if repo.join(glob_pattern).exists() {
            inv.contract_evidence.push(ContractEvidence {
                kind: kind.to_string(),
                path: glob_pattern.to_string(),
                status: "detected".to_string(),
            });
        }
    }
    if repo.join("contracts").exists() {
        inv.contract_evidence.push(ContractEvidence {
            kind: "directory".to_string(),
            path: "contracts/".to_string(),
            status: "detected".to_string(),
        });
    }
    if repo.join("schemas").exists() {
        inv.contract_evidence.push(ContractEvidence {
            kind: "directory".to_string(),
            path: "schemas/".to_string(),
            status: "detected".to_string(),
        });
    }

    inv
}

fn di(name: &str, evidence: &str, confidence: &str) -> DetectedItem {
    DetectedItem {
        name: name.to_string(),
        evidence: evidence.to_string(),
        confidence: confidence.to_string(),
    }
}

fn has_lockfile(repo: &Path) -> bool {
    repo.join("Cargo.lock").exists()
        || repo.join("package-lock.json").exists()
        || repo.join("pnpm-lock.yaml").exists()
        || repo.join("yarn.lock").exists()
        || repo.join("Gemfile.lock").exists()
        || repo.join("composer.lock").exists()
        || repo.join("go.sum").exists()
}

// ---------------------------------------------------------------------------
// Dimensional Liability Score
// ---------------------------------------------------------------------------

pub fn compute_liability(repo: &Path, inv: &StackInventory) -> LiabilityBreakdown {
    let mut dims = vec![];

    // 1. agent-operability
    let has_agents = repo.join("AGENTS.md").exists() || repo.join("agent").exists();
    let ao_score = if has_agents { 20 } else { 60 };
    dims.push(LiabilityDimension {
        name: "agent-operability".to_string(),
        score: ao_score,
        weight: 0.10,
        evidence: if has_agents {
            vec!["agent configuration detected".into()]
        } else {
            vec!["no agent configuration".into()]
        },
    });

    // 2. contract-drift
    let has_contracts = !inv.contract_evidence.is_empty();
    let cd_score = if has_contracts { 25 } else { 65 };
    dims.push(LiabilityDimension {
        name: "contract-drift".to_string(),
        score: cd_score,
        weight: 0.12,
        evidence: if has_contracts {
            vec![format!(
                "{} contract artifacts detected",
                inv.contract_evidence.len()
            )]
        } else {
            vec!["no contract artifacts detected".into()]
        },
    });

    // 3. product-truth-sprawl
    let multi_lang = inv.languages.len() > 2;
    let pts_score = if multi_lang {
        70
    } else if inv.languages.len() > 1 {
        45
    } else {
        25
    };
    dims.push(LiabilityDimension {
        name: "product-truth-sprawl".to_string(),
        score: pts_score,
        weight: 0.12,
        evidence: vec![format!("{} languages detected", inv.languages.len())],
    });

    // 4. security-risk
    let no_lock = !has_lockfile(repo);
    let sr_score = if no_lock { 70 } else { 30 };
    dims.push(LiabilityDimension {
        name: "security-risk".to_string(),
        score: sr_score,
        weight: 0.15,
        evidence: if no_lock {
            vec!["no lockfile detected".into()]
        } else {
            vec!["lockfile present".into()]
        },
    });

    // 5. db-data-risk
    let db_count = inv.db_clients.len();
    let ddr_score = if db_count > 1 {
        65
    } else if db_count == 1 {
        35
    } else {
        15
    };
    dims.push(LiabilityDimension {
        name: "db-data-risk".to_string(),
        score: ddr_score,
        weight: 0.13,
        evidence: vec![format!("{} DB client(s) detected", db_count)],
    });

    // 6. test-proof-gaps
    let has_tests = !inv.test_frameworks.is_empty();
    let has_ci = !inv.ci_systems.is_empty();
    let tpg_score = match (has_tests, has_ci) {
        (true, true) => 15,
        (true, false) => 40,
        (false, true) => 50,
        (false, false) => 80,
    };
    dims.push(LiabilityDimension {
        name: "test-proof-gaps".to_string(),
        score: tpg_score,
        weight: 0.15,
        evidence: vec![format!("tests={has_tests}, ci={has_ci}")],
    });

    // 7. runtime-cost-risk
    let is_rust = inv.languages.iter().any(|l| l.name == "rust");
    let rcr_score = if is_rust {
        15
    } else if inv.languages.iter().any(|l| l.name == "go") {
        25
    } else {
        50
    };
    dims.push(LiabilityDimension {
        name: "runtime-cost-risk".to_string(),
        score: rcr_score,
        weight: 0.10,
        evidence: vec![format!(
            "primary language: {}",
            inv.languages
                .first()
                .map(|l| l.name.as_str())
                .unwrap_or("unknown")
        )],
    });

    // 8. migration-complexity
    let fw_count = inv.frameworks.len();
    let mc_score = if fw_count > 2 {
        70
    } else if fw_count > 0 {
        40
    } else {
        55
    };
    dims.push(LiabilityDimension {
        name: "migration-complexity".to_string(),
        score: mc_score,
        weight: 0.13,
        evidence: vec![format!("{} framework(s) detected", fw_count)],
    });

    let total: f64 = dims.iter().map(|d| d.score as f64 * d.weight).sum();
    LiabilityBreakdown {
        total: total.round().clamp(0.0, 100.0) as u32,
        dimensions: dims,
    }
}

// ---------------------------------------------------------------------------
// Report Builder
// ---------------------------------------------------------------------------

pub fn build_migration_report(repo: &Path, target: &str) -> Result<MigrationReport> {
    let inv = detect_stack(repo);
    let liability = compute_liability(repo, &inv);

    let mut module_inventory = vec![];
    for item in &inv.languages {
        module_inventory.push(format!("language:{}", item.name));
    }
    for item in &inv.frameworks {
        module_inventory.push(format!("framework:{}", item.name));
    }
    for item in &inv.package_managers {
        module_inventory.push(format!("package-manager:{}", item.name));
    }

    let db_surfaces = if inv.db_clients.is_empty() {
        None
    } else {
        Some(
            inv.db_clients
                .iter()
                .map(|c| format!("db-client:{}", c.name))
                .collect(),
        )
    };

    let api_surfaces = if inv.api_surfaces.is_empty() {
        None
    } else {
        Some(
            inv.api_surfaces
                .iter()
                .map(|a| format!("api-framework:{}", a.framework))
                .collect(),
        )
    };

    let strangler_candidates = if inv.db_clients.is_empty() && inv.api_surfaces.is_empty() {
        None
    } else {
        let mut candidates = vec![];
        for db in &inv.db_clients {
            candidates.push(format!("isolate-db-layer:{}", db.name));
        }
        for api in &inv.api_surfaces {
            candidates.push(format!("isolate-api-surface:{}", api.framework));
        }
        Some(candidates)
    };

    let mut recommended_slice_order = vec![
        "inventory-and-classify".to_string(),
        "extract-contracts".to_string(),
    ];
    if !inv.db_clients.is_empty() {
        recommended_slice_order.push("isolate-db-adapter-layer".to_string());
    }
    recommended_slice_order.push("port-business-logic".to_string());
    recommended_slice_order.push("prove-equivalence".to_string());
    recommended_slice_order.push("cutover-and-retire".to_string());

    let required_proof_lanes = vec!["fast".to_string(), "contract".to_string()];

    let missing_tests = if inv.test_frameworks.is_empty() {
        Some(vec![
            "no test framework detected — migration risk is elevated".to_string(),
        ])
    } else {
        None
    };

    let high_risk_areas = if inv.ci_systems.is_empty() {
        Some(vec![
            "no CI system detected — migration cannot be verified automatically".to_string(),
        ])
    } else {
        None
    };

    let source_stack = {
        let langs: Vec<&str> = inv.languages.iter().map(|l| l.name.as_str()).collect();
        let fws: Vec<&str> = inv.frameworks.iter().map(|f| f.name.as_str()).collect();
        let mut parts = vec![];
        if !langs.is_empty() {
            parts.push(langs.join("+"));
        }
        if !fws.is_empty() {
            parts.push(fws.join("+"));
        }
        if parts.is_empty() {
            "unknown".to_string()
        } else {
            parts.join("/")
        }
    };

    Ok(MigrationReport {
        schema_version: "1.0.0".to_string(),
        command: "jankurai migrate".to_string(),
        status: "complete".to_string(),
        generated_at: now_string(),
        source_root: repo.display().to_string(),
        source_stack,
        target_stack: target.to_string(),
        liability_score: liability.total,
        liability_breakdown: liability,
        inventory: inv.clone(),
        module_inventory,
        owner_guesses: None,
        external_boundaries: None,
        db_surfaces,
        api_surfaces,
        duplicate_logic: None,
        high_risk_areas,
        missing_tests,
        strangler_candidates,
        contract_evidence: inv.contract_evidence,
        recommended_slice_order,
        required_proof_lanes,
        rollback_cutover_notes: vec![
            "migration planning is derived from repo inventory and audit evidence".to_string(),
            "each slice should have explicit rollback before cutover".to_string(),
        ],
    })
}

// ---------------------------------------------------------------------------
// Plan Builder
// ---------------------------------------------------------------------------

pub fn build_migration_plan(repo: &Path, target: &str) -> Result<MigrationPlan> {
    let report = build_migration_report(repo, target)?;

    let mut slices = vec![];
    let mut order: u32 = 1;

    // Generate one slice per DB surface
    if let Some(ref db_surfaces) = report.db_surfaces {
        for (i, db) in db_surfaces.iter().enumerate() {
            let risk = if report.liability_score > 60 {
                "high"
            } else {
                "medium"
            };
            slices.push(MigrationSlice {
                slice_id: format!("db-isolation-{}", i + 1),
                owner: "tools".to_string(),
                status: "candidate".to_string(),
                risk_level: risk.to_string(),
                dependency_order: order,
                human_approval_required: risk == "high",
                allowed_paths: vec![format!("crates/adapters/db/{}", db.replace("db-client:", ""))],
                forbidden_paths: vec!["crates/domain/".to_string()],
                contracts: vec!["adapter boundary interface".to_string()],
                tests: vec!["adapter integration test".to_string()],
                proof_lanes: vec!["db".to_string(), "fast".to_string()],
                rollback_notes: vec!["revert adapter extraction if contract tests fail".to_string()],
                cutover_notes: None,
                notes: Some(format!("isolate {db} behind adapter boundary")),
            });
            order += 1;
        }
    }

    // Generate one slice per API surface
    if let Some(ref api_surfaces) = report.api_surfaces {
        for (i, api) in api_surfaces.iter().enumerate() {
            slices.push(MigrationSlice {
                slice_id: format!("api-contract-{}", i + 1),
                owner: "tools".to_string(),
                status: "candidate".to_string(),
                risk_level: "medium".to_string(),
                dependency_order: order,
                human_approval_required: false,
                allowed_paths: vec!["contracts/".to_string()],
                forbidden_paths: vec![],
                contracts: vec!["OpenAPI or JSON Schema contract".to_string()],
                tests: vec!["consumer/provider contract test".to_string()],
                proof_lanes: vec!["contract".to_string(), "fast".to_string()],
                rollback_notes: vec![
                    "revert contract extraction if provider tests fail".to_string()
                ],
                cutover_notes: None,
                notes: Some(format!("extract contract for {api}")),
            });
            order += 1;
        }
    }

    // Always add a final equivalence proof slice
    slices.push(MigrationSlice {
        slice_id: "equivalence-proof".to_string(),
        owner: "tools".to_string(),
        status: if slices.is_empty() {
            "blocked".to_string()
        } else {
            "candidate".to_string()
        },
        risk_level: "high".to_string(),
        dependency_order: order,
        human_approval_required: true,
        allowed_paths: vec!["tests/equivalence/".to_string()],
        forbidden_paths: vec![],
        contracts: vec!["golden input/output equivalence".to_string()],
        tests: vec!["equivalence comparison test".to_string()],
        proof_lanes: vec!["fast".to_string()],
        rollback_notes: vec!["equivalence failures block cutover".to_string()],
        cutover_notes: Some(vec![
            "shadow reads recommended before cutover".to_string(),
            "parallel-run comparison for critical paths".to_string(),
        ]),
        notes: Some("prove equivalent behavior before retiring old code".to_string()),
    });

    let mut human_approvals = vec!["high-risk cutovers require human review".to_string()];
    if report.liability_score > 70 {
        human_approvals
            .push("liability score above 70 — all slices require human approval".to_string());
    }

    Ok(MigrationPlan {
        schema_version: "1.0.0".to_string(),
        command: "jankurai migrate".to_string(),
        status: "complete".to_string(),
        generated_at: now_string(),
        source_report: "target/jankurai/migration-report.json".to_string(),
        target_stack: report.target_stack.clone(),
        plan_mode: "dry-run".to_string(),
        slices,
        human_approval_requirements: human_approvals,
        commands: Some(vec![
            "jankurai migrate analyze . --json target/jankurai/migration-report.json".to_string(),
            "jankurai migrate plan . --json target/jankurai/migration-plan.json".to_string(),
        ]),
        warnings: if report.liability_score > 60 {
            Some(vec![format!(
                "liability score {} indicates elevated migration risk",
                report.liability_score
            )])
        } else {
            None
        },
    })
}

// ---------------------------------------------------------------------------
// CLI Entry Points
// ---------------------------------------------------------------------------

pub fn run(args: MigrateArgs) -> Result<()> {
    match args.mode {
        MigrateMode::Analyze => run_analyze(
            &args.repo,
            args.out.as_deref(),
            args.md.as_deref(),
            &args.target,
        ),
        MigrateMode::Plan => run_plan(
            &args.repo,
            args.out.as_deref(),
            args.md.as_deref(),
            &args.target,
        ),
    }
}

fn run_analyze(repo: &Path, out: Option<&str>, md: Option<&str>, target: &str) -> Result<()> {
    let report = build_migration_report(repo, target)?;
    if let Some(path) = out {
        validation::write_json(repo, ArtifactSchema::MigrationReport, path, &report)?;
    } else {
        println!("{}", serde_json::to_string_pretty(&report)?);
    }
    if let Some(path) = md {
        crate::render::write_markdown(path, &render_report_markdown(&report))?;
    }
    Ok(())
}

fn run_plan(repo: &Path, out: Option<&str>, md: Option<&str>, target: &str) -> Result<()> {
    let plan = build_migration_plan(repo, target)?;
    if let Some(path) = out {
        validation::write_json(repo, ArtifactSchema::MigrationPlan, path, &plan)?;
    } else {
        println!("{}", serde_json::to_string_pretty(&plan)?);
    }
    if let Some(path) = md {
        crate::render::write_markdown(path, &render_plan_markdown(&plan))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Markdown Rendering
// ---------------------------------------------------------------------------

fn render_report_markdown(report: &MigrationReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Migration Report");
    let _ = writeln!(out);
    let _ = writeln!(out, "- source stack: `{}`", report.source_stack);
    let _ = writeln!(out, "- target stack: `{}`", report.target_stack);
    let _ = writeln!(out, "- liability score: `{}`", report.liability_score);
    let _ = writeln!(
        out,
        "- module inventory: `{}`",
        report.module_inventory.join(", ")
    );
    if let Some(ref db) = report.db_surfaces {
        let _ = writeln!(out, "- DB surfaces: `{}`", db.join(", "));
    }
    if let Some(ref api) = report.api_surfaces {
        let _ = writeln!(out, "- API surfaces: `{}`", api.join(", "));
    }
    if let Some(ref strangler) = report.strangler_candidates {
        let _ = writeln!(out, "- strangler candidates: `{}`", strangler.join(", "));
    }
    let _ = writeln!(
        out,
        "- recommended slice order: `{}`",
        report.recommended_slice_order.join(" → ")
    );
    let _ = writeln!(
        out,
        "- required proof lanes: `{}`",
        report.required_proof_lanes.join(", ")
    );
    let _ = writeln!(
        out,
        "- rollback notes: `{}`",
        report.rollback_cutover_notes.join("; ")
    );
    out
}

fn render_plan_markdown(plan: &MigrationPlan) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Migration Plan");
    let _ = writeln!(out);
    let _ = writeln!(out, "- target stack: `{}`", plan.target_stack);
    let _ = writeln!(out, "- plan mode: `{}`", plan.plan_mode);
    let _ = writeln!(out, "- slices: `{}`", plan.slices.len());
    let _ = writeln!(out);
    for slice in &plan.slices {
        let _ = writeln!(out, "## Slice: {}", slice.slice_id);
        let _ = writeln!(out, "- owner: `{}`", slice.owner);
        let _ = writeln!(out, "- status: `{}`", slice.status);
        let _ = writeln!(out, "- proof lanes: `{}`", slice.proof_lanes.join(", "));
        if let Some(ref notes) = slice.notes {
            let _ = writeln!(out, "- notes: {notes}");
        }
        let _ = writeln!(out);
    }
    let _ = writeln!(out, "## Human Approval Requirements");
    for req in &plan.human_approval_requirements {
        let _ = writeln!(out, "- {req}");
    }
    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
