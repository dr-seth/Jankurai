// Symbol-mismatch + use-site-not-def-site verification — ARY-2029.
//
// The pre-existing path-line verifier in `verify-prompt` reported "verified"
// whenever *any* non-comment code lived at the claimed line. A documentation
// claim like "`_sense_formalize` at `engine.py:674`" then passed silently
// even when the actual symbol at L674 was `_decide` — the file-line
// re-derivation alone is not strong enough to falsify a wrong-symbol claim.
//
// The QO-side fixtures in `tests/fixtures/verify-prompt/` are the
// adversarial denial bench for this gap: `negative_v1.md` makes 5
// backtick-anchored path-line claims, 2 of which point at the right line but
// the wrong symbol (`_decide_route_via_llm` and `_verify_with_llm_judge`).
// `positive_v2.md` removes those wrong-symbol claims and should pass.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_jankurai"))
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/verify-prompt")
}

fn build_temp_repo(doc_name: &str) -> (tempfile::TempDir, PathBuf) {
    let repo_dir = tempdir().unwrap();
    for stub in ["synthetic_engine_stub.py", "synthetic_aara_api_stub.py"] {
        fs::copy(fixture_root().join(stub), repo_dir.path().join(stub)).unwrap();
    }
    fs::copy(
        fixture_root().join(doc_name),
        repo_dir.path().join(doc_name),
    )
    .unwrap();
    let path = repo_dir.path().to_path_buf();
    (repo_dir, path)
}

fn run_verb(
    repo: &PathBuf,
    doc: &str,
    strict: bool,
) -> (std::process::Output, tempfile::TempDir, PathBuf) {
    let out_dir = tempdir().unwrap();
    let json_path = out_dir.path().join("prompt.json");
    let mut cmd = Command::new(binary_path());
    cmd.arg("migrate")
        .arg(repo)
        .arg("verify-prompt")
        .arg(doc)
        .arg("--out")
        .arg(&json_path);
    if strict {
        cmd.arg("--strict");
    }
    let output = cmd.output().unwrap();
    (output, out_dir, json_path)
}

#[test]
fn negative_v1_surfaces_symbol_mismatches() {
    let (_keep, repo) = build_temp_repo("negative_v1.md");
    let (output, _out_keep, json_path) = run_verb(&repo, "negative_v1.md", false);
    assert!(
        output.status.success(),
        "verb crashed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap();
    assert_eq!(report["decision"], "fail", "negative fixture must fail");

    // Re-derived expectations: the two wrong-symbol claims should land as
    // invalid with note "symbol mismatch", not pass silently.
    let claims = report["claims"].as_array().unwrap();
    let mismatches: Vec<&serde_json::Value> = claims
        .iter()
        .filter(|c| c["note"].as_str() == Some("symbol mismatch"))
        .collect();
    assert!(
        mismatches.len() >= 2,
        "expected ≥2 symbol-mismatch findings, got {}: {}",
        mismatches.len(),
        serde_json::to_string_pretty(&claims).unwrap()
    );

    // The single true claim (`_act` at L1171) should still verify — proving
    // the symbol check discriminates rather than blanket-failing every
    // path-line claim with a backtick token.
    let act_verified = claims.iter().any(|c| {
        c["claim"].as_str() == Some("synthetic_engine_stub.py:1171")
            && c["decision"].as_str() == Some("verified")
    });
    assert!(
        act_verified,
        "the true `_act` claim at L1171 should still verify: {}",
        serde_json::to_string_pretty(&claims).unwrap()
    );
}

#[test]
fn positive_v2_passes_with_symbol_check() {
    let (_keep, repo) = build_temp_repo("positive_v2.md");
    let (output, _out_keep, json_path) = run_verb(&repo, "positive_v2.md", true);
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap();
    assert!(
        output.status.success(),
        "v2 must pass even with --strict (decision={}, stderr={}): {}",
        report["decision"],
        String::from_utf8_lossy(&output.stderr),
        serde_json::to_string_pretty(&report).unwrap()
    );

    // None of the v2 claims should land as a symbol mismatch — v2 either
    // names the correct symbol or omits the symbol annotation entirely.
    let claims = report["claims"].as_array().unwrap();
    let mismatches: usize = claims
        .iter()
        .filter(|c| c["note"].as_str() == Some("symbol mismatch"))
        .count();
    assert_eq!(
        mismatches,
        0,
        "v2 should produce zero symbol-mismatch findings: {}",
        serde_json::to_string_pretty(&claims).unwrap()
    );
}
