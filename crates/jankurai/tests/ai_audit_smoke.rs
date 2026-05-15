// `jankurai ai audit` — fixture smoke + Rule-8/9 adversarial denial suite
// (ARY-2030, verb #2).
//
// Rule 9: every assertion re-derives evidence from the JSON report the verb
// emits (`--out`), not by regex-matching the stdout banner. The classifier
// must prove a tier from import-parse + call-shape, never from an SDK name
// appearing as a string literal.
//
// Rule 8: the adversarial fixtures (`oauth_falsepositive.py`,
// `llm_settings_falsepositive.py`, `dangling_import_adversarial.py`) are the
// denial bench — the verb MUST refuse to classify them as replaceable LLM
// call sites.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_jankurai"))
}

fn fixture_dir(sub: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ai-audit")
        .join(sub)
}

fn run_audit(target: &PathBuf) -> serde_json::Value {
    let out_dir = tempdir().unwrap();
    let json_path = out_dir.path().join("ai-audit.json");
    let output = Command::new(binary_path())
        .arg("ai")
        .arg("audit")
        .arg(target)
        .arg("--out")
        .arg(&json_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "verb exited non-zero: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap()
}

/// Map file -> (shape, tier) re-derived from the report.
fn site_map(report: &serde_json::Value) -> HashMap<String, (String, Option<String>)> {
    report["sites"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (
                s["file"].as_str().unwrap().to_string(),
                (
                    s["shape"].as_str().unwrap().to_string(),
                    s["tier"].as_str().map(str::to_string),
                ),
            )
        })
        .collect()
}

#[test]
fn seven_fixtures_classify_to_expected_tiers() {
    let report = run_audit(&fixture_dir("source_fixtures"));
    let sites = site_map(&report);

    let expect: &[(&str, &str, &str)] = &[
        ("trading_nl_classify.py", "CLASSIFY", "tier_2"),
        ("trading_nl_generate.py", "GENERATE", "tier_3"),
        ("channel_wrapper.py", "WRAPPER", "tier_2"),
        ("meta_rsi_agent.py", "AGENT", "tier_3"),
        ("integrated_module.py", "INTEGRATED", "tier_1"),
        ("oauth_falsepositive.py", "FALSE_POSITIVE", "tier_4"),
        ("llm_settings_falsepositive.py", "FALSE_POSITIVE", "tier_4"),
    ];

    for (file, shape, tier) in expect {
        let got = sites
            .get(*file)
            .unwrap_or_else(|| panic!("no site for {file}: {sites:?}"));
        assert_eq!(
            got.0,
            *shape,
            "{file}: expected shape {shape}, got {} (report: {})",
            got.0,
            serde_json::to_string_pretty(&report).unwrap()
        );
        assert_eq!(
            got.1.as_deref(),
            Some(*tier),
            "{file}: expected {tier}, got {:?}",
            got.1
        );
    }

    // Per-tier file counts re-derived from the report fields.
    assert_eq!(report["tier1_files"], 1);
    assert_eq!(report["tier2_files"], 2);
    assert_eq!(report["tier3_files"], 2);
    assert_eq!(report["tier4_files"], 2);
}

#[test]
fn expected_output_summary_lines_present() {
    // The asserter spec (expected_output.txt) requires these exact lines.
    let out_dir = tempdir().unwrap();
    let json_path = out_dir.path().join("o.json");
    let stdout = Command::new(binary_path())
        .arg("ai")
        .arg("audit")
        .arg(fixture_dir("source_fixtures"))
        .arg("--out")
        .arg(&json_path)
        .output()
        .unwrap()
        .stdout;
    let text = String::from_utf8_lossy(&stdout);
    for needle in [
        "Tier 1 (already integrated):  1 file",
        "Tier 2 (replaceable):",
        "Tier 3 (permanent):",
        "Tier 4 (false positive):      2 files",
        "CLASSIFY",
        "tier_2",
        "GENERATE",
        "tier_3",
        "AGENT",
        "INTEGRATED",
        "tier_1",
        "FALSE_POSITIVE",
        "tier_4",
    ] {
        assert!(
            text.contains(needle),
            "stdout missing required line `{needle}`:\n{text}"
        );
    }
}

#[test]
fn rule9_oauth_falsepositive_is_tier4_not_a_call_site() {
    // String "anthropic.com" present, NO `import anthropic`. A regex-only
    // auditor misclassifies this as a live site; AST/import proof => tier_4.
    let report = run_audit(&fixture_dir("source_fixtures"));
    let sites = site_map(&report);
    let (shape, tier) = sites.get("oauth_falsepositive.py").unwrap();
    assert_eq!(shape, "FALSE_POSITIVE");
    assert_eq!(tier.as_deref(), Some("tier_4"));
}

#[test]
fn rule9_settings_keys_are_not_call_sites() {
    let report = run_audit(&fixture_dir("source_fixtures"));
    let sites = site_map(&report);
    let (shape, tier) = sites.get("llm_settings_falsepositive.py").unwrap();
    assert_eq!(shape, "FALSE_POSITIVE");
    assert_eq!(tier.as_deref(), Some("tier_4"));
}

#[test]
fn rule8_dangling_import_yields_no_call_site_not_a_tier() {
    // Adversarial: imports anthropic, never calls `.messages.create`. The
    // verb MUST NOT invent a replaceable site; it emits `no_call_site` with
    // NO tier (excluded from the tier-1..4 summary counts).
    let report = run_audit(&fixture_dir("adversarial"));
    let sites = site_map(&report);
    let (shape, tier) = sites
        .get("dangling_import_adversarial.py")
        .expect("dangling-import file should appear in report");
    assert_eq!(shape, "no_call_site");
    assert_eq!(*tier, None, "no_call_site must carry no tier");

    // It must not inflate any replaceable-tier count.
    assert_eq!(report["tier1_files"], 0);
    assert_eq!(report["tier2_files"], 0);
    assert_eq!(report["tier3_files"], 0);
    assert_eq!(report["tier4_files"], 0);
}
