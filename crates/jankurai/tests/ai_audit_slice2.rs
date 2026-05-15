// ai-audit slice 2 (ARY-2030): OpenAI-flavor CLASSIFY + WRAPPER-vs-GENERATE
// disambiguation + `jankurai doctor` config-hygiene integration.
//
// Rule 9: every classification assertion is re-derived from the JSON
// report the verb emits, not the stdout banner.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_jankurai"))
}

fn fixture(p: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ai-audit")
        .join(p)
}

fn audit(dir: &PathBuf) -> serde_json::Value {
    let out_dir = tempdir().unwrap();
    let json_path = out_dir.path().join("ai-audit.json");
    let out = Command::new(binary())
        .arg("ai")
        .arg("audit")
        .arg(dir)
        .arg("--out")
        .arg(&json_path)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "verb failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap()
}

fn site(report: &serde_json::Value, file: &str) -> (String, Option<String>) {
    report["sites"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["file"].as_str() == Some(file))
        .map(|s| {
            (
                s["shape"].as_str().unwrap().to_string(),
                s["tier"].as_str().map(str::to_string),
            )
        })
        .unwrap_or_else(|| panic!("no site for {file}: {report}"))
}

#[test]
fn openai_flavor_response_format_is_classify_tier2() {
    // OpenAI structured output uses response_format={"type":"json_object"},
    // not a prose JSON-shape instruction. The CLASSIFY heuristic must
    // accept that branch.
    let report = audit(&fixture("slice2"));
    let (shape, tier) = site(&report, "trading_nl_classify_openai.py");
    assert_eq!(
        shape, "CLASSIFY",
        "OpenAI flavor must be CLASSIFY: {report}"
    );
    assert_eq!(tier.as_deref(), Some("tier_2"));
}

#[test]
fn wrapper_dressed_as_generate_resolves_to_wrapper_not_generate() {
    // max_tokens=2000 (>= generate_min) but a closed enumerated set in the
    // system prompt: the enum heuristic must win → WRAPPER (tier_2), never
    // GENERATE (tier_3).
    let report = audit(&fixture("slice2"));
    let (shape, tier) = site(&report, "wrapper_dressed_as_generate.py");
    assert_eq!(
        shape, "WRAPPER",
        "closed-set + big max_tokens must disambiguate to WRAPPER: {report}"
    );
    assert_eq!(tier.as_deref(), Some("tier_2"));
}

#[test]
fn slice2_dir_is_all_tier2_no_false_negatives() {
    let report = audit(&fixture("slice2"));
    assert_eq!(report["tier2_files"], 2, "{report}");
    assert_eq!(report["tier3_files"], 0, "no GENERATE leak: {report}");
    assert_eq!(report["tier4_files"], 0, "{report}");
}

// --- doctor config-hygiene integration ---

fn run_doctor(repo: &std::path::Path) -> (bool, String) {
    let json = repo.join("doc.json");
    let out = Command::new(binary())
        .arg("doctor")
        .arg(repo)
        .arg("--json")
        .arg(&json)
        .arg("--fail-on")
        .arg("critical")
        .output()
        .unwrap();
    let body = fs::read_to_string(&json).unwrap_or_default();
    (out.status.success(), body)
}

#[test]
fn doctor_flags_broken_ai_audit_config_but_ignores_absent_one() {
    // Absent config: doctor must NOT invent a diagnostic.
    let clean = tempdir().unwrap();
    let (_ok, body) = run_doctor(clean.path());
    assert!(
        !body.contains("ai-audit-config"),
        "absent ai-audit.toml should be silent: {body}"
    );

    // Present but broken (wrong scalar type): doctor must flag it.
    let bad = tempdir().unwrap();
    fs::create_dir_all(bad.path().join("agent")).unwrap();
    fs::write(
        bad.path().join("agent/ai-audit.toml"),
        "[heuristics]\nclassify_max_tokens_max = \"not-an-int\"\n",
    )
    .unwrap();
    let (_ok2, body2) = run_doctor(bad.path());
    assert!(
        body2.contains("ai-audit-config-schema")
            && body2.contains("classify_max_tokens_max must be an integer"),
        "broken ai-audit config not flagged: {body2}"
    );

    // Present and well-formed: no ai-audit diagnostic.
    let good = tempdir().unwrap();
    fs::create_dir_all(good.path().join("agent")).unwrap();
    fs::write(
        good.path().join("agent/ai-audit.toml"),
        "[heuristics]\nclassify_max_tokens_max = 500\nclassify_temperature_max = 0.3\ngenerate_max_tokens_min = 1000\n",
    )
    .unwrap();
    let (_ok3, body3) = run_doctor(good.path());
    assert!(
        !body3.contains("ai-audit-config-schema"),
        "well-formed config wrongly flagged: {body3}"
    );
}
