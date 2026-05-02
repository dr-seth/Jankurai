use super::helpers::*;
use super::rules;
use crate::model::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct FindingBuilder<'a> {
    ctx: &'a AuditContext,
    findings: Vec<Finding>,
    has_any_finding: bool,
    has_context_finding: bool,
}

impl<'a> FindingBuilder<'a> {
    pub fn new(ctx: &'a AuditContext) -> Self {
        Self {
            ctx,
            findings: Vec::new(),
            has_any_finding: false,
            has_context_finding: false,
        }
    }

    pub fn add(
        &mut self,
        severity: &str,
        category: &str,
        path: &str,
        problem: &str,
        fix: &str,
        evidence: Vec<String>,
        rule_id: Option<&str>,
        line: Option<usize>,
    ) {
        if category == "context" {
            self.has_context_finding = true;
        }
        self.has_any_finding = true;
        let rule = rule_id.unwrap_or("HLT-000-SCORE-DIMENSION");
        let rule_meta = rule_id.and_then(rules::lookup);
        let lane = rule_meta
            .map(|rule| rule.lane)
            .or_else(|| lane_for(category))
            .map(|s| s.into());
        let owner = owner_for_path(self.ctx, path).or_else(|| {
            rule_meta
                .filter(|rule| !rule.owner_hint.is_empty())
                .map(|rule| rule.owner_hint.to_string())
        });
        let evidence_kind = rule_meta
            .map(|rule| rule.evidence_kind)
            .unwrap_or_else(|| evidence_kind_for_path(path));
        let fingerprint = finding_fingerprint(rule, category, path, problem, &evidence);
        self.findings.push(Finding {
            severity: severity.into(),
            category: category.into(),
            path: path.into(),
            problem: problem.into(),
            agent_fix: fix.into(),
            evidence,
            check_id: format!("{rule}:{category}"),
            hardness: hardness_for_severity(severity).into(),
            confidence: confidence_for_severity(severity),
            evidence_kind: evidence_kind.into(),
            rerun_command: rerun_command_for_lane(lane.as_deref()).into(),
            fingerprint,
            rule_id: rule_id.map(|s| s.into()),
            tlr: rule_meta
                .map(|rule| rule.tlr)
                .or_else(|| tlr_for(category))
                .map(|s| s.into()),
            lane,
            docs_url: rule_id.and_then(rules::docs_for_rule_id).map(|s| s.into()),
            owner,
            line,
            matched_term: None,
            reason: None,
        })
    }

    pub fn has_any_finding(&self) -> bool {
        self.has_any_finding
    }

    pub fn has_context_finding(&self) -> bool {
        self.has_context_finding
    }

    pub fn into_findings(self) -> Vec<Finding> {
        self.findings
    }
}

pub fn hardness_for_severity(severity: &str) -> &'static str {
    match severity {
        "critical" | "high" => "hard",
        _ => "soft",
    }
}

pub fn confidence_for_severity(severity: &str) -> f64 {
    match severity {
        "critical" => 0.95,
        "high" => 0.88,
        "medium" => 0.76,
        _ => 0.62,
    }
}

pub fn evidence_kind_for_path(path: &str) -> &'static str {
    if path.starts_with(".github/workflows") {
        "workflow-command"
    } else if path.starts_with("agent/") {
        "policy-manifest"
    } else if path.starts_with("docs/") {
        "documentation"
    } else {
        "repository-scan"
    }
}

pub fn rerun_command_for_lane(lane: Option<&str>) -> &'static str {
    match lane.unwrap_or("audit") {
        "security" => "just security",
        "contract" => "just fast",
        "db" => "just fast",
        "web" | "e2e" => "just ux-qa",
        "fast" => "just fast",
        "release" => "just check",
        _ => "just score",
    }
}

pub fn finding_fingerprint(
    rule_id: &str,
    category: &str,
    path: &str,
    problem: &str,
    evidence: &[String],
) -> String {
    let mut hasher = DefaultHasher::new();
    rule_id.hash(&mut hasher);
    category.hash(&mut hasher);
    path.hash(&mut hasher);
    problem.hash(&mut hasher);
    for item in evidence.iter().take(3) {
        item.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

pub fn tlr_for(category: &str) -> Option<&'static str> {
    Some(match category {
        "security" => "Security",
        "python" => "Business truth",
        "data" => "Contracts/data",
        "test" => "Verification",
        "audit" => "Context/setup",
        "generated" => "Contracts/data",
        "boundary" => "Contracts/data",
        "context" => "Context/setup",
        "vibe" => "Entropy",
        "shape" => "Entropy",
        "observability" => "Repair",
        "docs" => "Context/setup",
        "proof" => "Verification",
        "stack" => "Context/setup",
        "ux-qa" => "Verification and rendered UX",
        "exceptions" => "Repair",
        _ => return None,
    })
}

pub fn lane_for(category: &str) -> Option<&'static str> {
    Some(match category {
        "security" => "security",
        "python" => "contract",
        "data" => "db",
        "test" => "fast",
        "audit" => "audit",
        "generated" => "contract",
        "boundary" => "contract",
        "context" => "fast",
        "vibe" => "fast",
        "shape" => "fast",
        "observability" => "observability",
        "docs" => "audit",
        "proof" => "fast",
        "stack" => "audit",
        "ux-qa" => "web",
        "exceptions" => "observability",
        _ => return None,
    })
}

fn owner_for_path(ctx: &AuditContext, rel_path: &str) -> Option<String> {
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Deserialize)]
    struct OwnerMapFile {
        owners: HashMap<String, String>,
    }

    if let Ok(text) = std::fs::read_to_string(ctx.root.join("agent/owner-map.json")) {
        if let Ok(parsed) = serde_json::from_str::<OwnerMapFile>(&text) {
            for (prefix, owner) in parsed.owners.iter() {
                if rel_path == prefix || rel_path.starts_with(prefix) {
                    return Some(owner.clone());
                }
            }
        }
    }
    for (prefix, owner) in OWNER_MAP_PREFIXES {
        if rel_path == *prefix || rel_path.starts_with(prefix) {
            return Some((*owner).to_string());
        }
    }
    None
}

pub fn dimension_soft_route(
    name: &str,
) -> (&'static str, &'static str, &'static str, &'static str) {
    match name {
        "Ownership and navigation surface" => (
            "context",
            "agent/owner-map.json",
            "HLT-003-OWNERLESS-PATH",
            "tighten owner/test maps and root routing until agents can localize ownership without inference",
        ),
        "Contract and boundary integrity" => (
            "boundary",
            "agent/boundaries.toml",
            "HLT-007-HANDWRITTEN-CONTRACT",
            "add generated contracts and boundary checks for public APIs, data access, and cross-runtime seams",
        ),
        "Proof lanes and test routing" => (
            "proof",
            "agent/test-map.json",
            "HLT-004-UNMAPPED-PROOF",
            "route each owned path to a deterministic proof command and make the lane executable in CI",
        ),
        "Security and supply-chain posture" => (
            "security",
            ".github/workflows/jankurai.yml",
            "HLT-016-SUPPLY-CHAIN-DRIFT",
            "wire secret, dependency, provenance, and workflow scans into an operational CI lane",
        ),
        "Code shape and semantic surface" => (
            "shape",
            ".",
            "HLT-001-DEAD-MARKER",
            "split large or ambiguous authored code into smaller semantic modules with focused tests",
        ),
        "Data truth and workflow safety" => (
            "data",
            "db/",
            "HLT-006-DIRECT-DB-WRONG-LAYER",
            "move durable truth into migrations, constraints, adapters, and application-owned transactions",
        ),
        "Observability and repair evidence" => (
            "observability",
            "docs/testing.md",
            "HLT-017-OPAQUE-OBSERVABILITY",
            "add structured errors, telemetry, and repair receipts that tell the next agent where to rerun proof",
        ),
        "Context economy and agent instructions" => (
            "context",
            "AGENTS.md",
            "HLT-015-CONTEXT-SETUP-GAP",
            "keep root guidance short and route durable detail through agent-readable manifests and docs",
        ),
        "Python containment and polyglot hygiene" => (
            "python",
            "python/ai-service",
            "HLT-005-PYTHON-PRODUCT-TRUTH",
            "keep Python bounded to AI/data tooling and move product truth into Rust, SQL, and generated contracts",
        ),
        "Build speed signals" => (
            "proof",
            "Justfile",
            "HLT-018-PERF-CONCURRENCY-DRIFT",
            "add fast deterministic build/test targets, caches, and narrow proof lanes for agent iteration",
        ),
        _ => (
            "audit",
            "agent/audit-policy.toml",
            "HLT-017-OPAQUE-OBSERVABILITY",
            "add a rule-specific repair route for this below-floor dimension",
        ),
    }
}
