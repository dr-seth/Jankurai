use super::helpers::*;
use super::scan;

pub const CAPS: &[(&str, i32)] = &[
    ("no-root-agent-instructions", 75),
    ("no-one-command-setup-or-validation", 70),
    ("no-deterministic-fast-lane", 65),
    ("no-security-lane-on-high-risk-repo", 60),
    ("generated-contracts-or-public-api-drift-untested", 80),
    ("python-direct-product-truth-or-db-ownership", 72),
    ("no-secret-or-dependency-scanning-in-ci", 78),
    ("no-jankurai-audit-lane-in-ci", 82),
    ("non-optimal-product-language-found", 74),
    ("too-much-python-in-product-surface", 72),
    ("vibe-placeholders-in-product-code", 68),
    ("fallback-soup-in-product-code", 70),
    ("future-hostile-dead-language-in-product-code", 64),
    ("severe-duplication-in-product-code", 70),
    ("generated-zone-mutation-risk", 76),
    ("direct-db-access-from-wrong-layer", 66),
    ("missing-web-e2e-lane", 82),
    ("missing-rendered-ux-qa-lane", 84),
    ("prompt-injection-risk", 78),
    ("overbroad-agent-agency", 65),
    ("secret-like-content-detected", 60),
    ("false-green-test-risk", 76),
    ("destructive-migration-risk", 70),
    ("missing-rust-property-or-integration-tests", 82),
    ("no-agent-friendly-exception-pattern", 76),
    ("missing-agent-readable-docs", 80),
    ("streaming-runtime-drift", 78),
];

pub fn caps_applied(ctx: &AuditContext, has_destructive_migration_sql: bool) -> Vec<String> {
    let mut caps = Vec::new();
    if !has_root_agents(ctx) {
        caps.push("no-root-agent-instructions".into());
    }
    if !has_one_command(ctx) {
        caps.push("no-one-command-setup-or-validation".into());
    }
    if !has_fast_lane(ctx) {
        caps.push("no-deterministic-fast-lane".into());
    }
    if is_high_risk_repo(ctx) && !has_security_lane(ctx) {
        caps.push("no-security-lane-on-high-risk-repo".into());
    }
    if (has_contract_surface(ctx) || has_polyglot_boundary(ctx))
        && !(has_generated_contracts(ctx) || has_api_drift_checks(ctx))
    {
        caps.push("generated-contracts-or-public-api-drift-untested".into());
    }
    if bad_python_paths(ctx) {
        caps.push("python-direct-product-truth-or-db-ownership".into());
    }
    if is_high_risk_repo(ctx) && !has_secret_or_dependency_scans(ctx) {
        caps.push("no-secret-or-dependency-scanning-in-ci".into());
    }
    if is_high_risk_repo(ctx) && !has_jankurai_audit_ci_lane(ctx) {
        caps.push("no-jankurai-audit-lane-in-ci".into());
    }
    if !non_optimal_language_hits(ctx).is_empty() {
        caps.push("non-optimal-product-language-found".into());
    }
    if python_ratio(ctx) > 0.15 {
        caps.push("too-much-python-in-product-surface".into());
    }
    if !scan::todo_hits(ctx).is_empty() {
        caps.push("vibe-placeholders-in-product-code".into());
    }
    if scan::fallback_hits(ctx).len() > 1 {
        caps.push("fallback-soup-in-product-code".into());
    }
    if !scan::future_hostile_hits(ctx).is_empty() {
        caps.push("future-hostile-dead-language-in-product-code".into());
    }
    if !scan::duplicate_blocks(ctx).is_empty() {
        caps.push("severe-duplication-in-product-code".into());
    }
    if !scan::generated_zone_issues(ctx).is_empty()
        || !scan::generated_zone_manifest_metadata_issues(ctx).is_empty()
    {
        caps.push("generated-zone-mutation-risk".into());
    }
    if !scan::wrong_layer_db_hits(ctx).is_empty() {
        caps.push("direct-db-access-from-wrong-layer".into());
    }
    if has_web_surface(ctx) && !has_playwright_e2e(ctx) {
        caps.push("missing-web-e2e-lane".into());
    }
    if has_web_surface(ctx) && !super::analyzers::ux_qa_status(ctx).has_rendered_ux_lane {
        caps.push("missing-rendered-ux-qa-lane".into());
    }
    if !scan::prompt_injection_hits(ctx).is_empty() {
        caps.push("prompt-injection-risk".into());
    }
    if !scan::agency_hits(ctx).is_empty() {
        caps.push("overbroad-agent-agency".into());
    }
    if !scan::secret_hits(ctx).is_empty() {
        caps.push("secret-like-content-detected".into());
    }
    if !scan::false_green_hits(ctx).is_empty() {
        caps.push("false-green-test-risk".into());
    }
    if has_destructive_migration_sql {
        caps.push("destructive-migration-risk".into());
    }
    if has_rust_surface(ctx) && (!has_rust_property_tests(ctx) || !has_rust_integration_tests(ctx))
    {
        caps.push("missing-rust-property-or-integration-tests".into());
    }
    if !has_agent_friendly_exceptions(ctx) && !product_code_files(ctx).is_empty() {
        caps.push("no-agent-friendly-exception-pattern".into());
    }
    if !missing_core_docs(ctx).is_empty() {
        caps.push("missing-agent-readable-docs".into());
    }
    if !scan::streaming_runtime_hits(ctx).is_empty() {
        caps.push("streaming-runtime-drift".into());
    }
    caps
}
