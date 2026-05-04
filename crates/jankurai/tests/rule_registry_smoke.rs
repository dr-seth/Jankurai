use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use jankurai::audit::{rule_registry, rules};

#[test]
fn rule_registry_ids_are_unique() {
    let mut seen = HashSet::new();
    for rule in rule_registry() {
        assert!(
            seen.insert(rule.id),
            "duplicate rule id in registry: {}",
            rule.id
        );
    }
}

#[test]
fn hlt014_a11y_gap_is_registered() {
    let rule = rules::lookup("HLT-014-A11Y-GAP").expect("HLT-014-A11Y-GAP must exist in registry");
    assert_eq!(rule.id, "HLT-014-A11Y-GAP");
    assert_eq!(rule.category, "ux-qa");
    assert_eq!(rule.lane, "web");
}

#[test]
fn hlt021_destructive_migration_is_registered() {
    let rule = rules::lookup("HLT-021-DESTRUCTIVE-MIGRATION")
        .expect("HLT-021-DESTRUCTIVE-MIGRATION must exist in registry");
    assert_eq!(rule.id, "HLT-021-DESTRUCTIVE-MIGRATION");
    assert_eq!(rule.category, "data");
    assert_eq!(rule.lane, "db-migration-analyze");
}

#[test]
fn every_rule_has_repair_policy_metadata() {
    for rule in rules::all() {
        assert!(
            !rule.repair_reason.trim().is_empty(),
            "{} has empty repair reason",
            rule.id
        );
        assert!(
            matches!(
                rule.repair_eligibility.as_str(),
                "auto-safe" | "agent-assisted" | "human-required" | "never-auto"
            ),
            "{} has invalid repair eligibility",
            rule.id
        );
        assert!(
            matches!(
                rule.repair_risk.as_str(),
                "low" | "medium" | "high" | "critical"
            ),
            "{} has invalid repair risk",
            rule.id
        );
    }
}

#[test]
fn every_rule_id_is_documented_in_the_standard() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let standard = fs::read_to_string(repo.join("docs/agent-native-standard.md")).unwrap();
    let brief = fs::read_to_string(repo.join("agent/JANKURAI_STANDARD.md")).unwrap();
    for rule in rules::all() {
        assert!(
            standard.contains(rule.id),
            "{} missing from docs/agent-native-standard.md",
            rule.id
        );
        assert!(
            brief.contains(rule.id),
            "{} missing from agent/JANKURAI_STANDARD.md",
            rule.id
        );
    }
}
