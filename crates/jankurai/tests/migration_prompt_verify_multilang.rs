// Multi-language symbol resolver (TS / JS class methods + top-level
// functions + arrow bindings) — ARY-2029 slice 2.
//
// Slice 1 added `expected_symbol` / `verify_path_line` symbol-mismatch
// detection but only covered Python `def`/`class` and Rust `fn` headers.
// TypeScript class methods (`async foo() {}`, `public bar() {}`) silently
// fell through to "verified" because no def/class keyword was at the
// resolved line. Slice 2 dispatches by file extension and adds TS regexes
// for class-method, top-level-function, and arrow-binding shapes.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_jankurai"))
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/verify-prompt-multilang")
}

fn build_temp_repo(doc_name: &str) -> (tempfile::TempDir, PathBuf) {
    let repo_dir = tempdir().unwrap();
    fs::copy(
        fixture_root().join("synthetic_aara_engine_stub.ts"),
        repo_dir.path().join("synthetic_aara_engine_stub.ts"),
    )
    .unwrap();
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
fn ts_negative_v1_surfaces_class_method_and_fn_mismatches() {
    let (_keep, repo) = build_temp_repo("negative_v1_ts.md");
    let (output, _out_keep, json_path) = run_verb(&repo, "negative_v1_ts.md", false);
    assert!(
        output.status.success(),
        "verb crashed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap();
    assert_eq!(report["decision"], "fail", "TS negative fixture must fail");

    let claims = report["claims"].as_array().unwrap();

    // Five wrong-symbol claims (L20, L30, L42, L51, L57) — all must
    // land as symbol mismatch. L36 is the one true `act` claim.
    let mismatches: Vec<&serde_json::Value> = claims
        .iter()
        .filter(|c| c["note"].as_str() == Some("symbol mismatch"))
        .collect();
    assert!(
        mismatches.len() >= 5,
        "expected ≥5 TS symbol-mismatch findings, got {}: {}",
        mismatches.len(),
        serde_json::to_string_pretty(&claims).unwrap()
    );

    // The true `act` claim at L36 should still verify.
    let act_verified = claims.iter().any(|c| {
        c["claim"].as_str() == Some("synthetic_aara_engine_stub.ts:36")
            && c["decision"].as_str() == Some("verified")
    });
    assert!(
        act_verified,
        "true `act` claim at L36 must still verify: {}",
        serde_json::to_string_pretty(&claims).unwrap()
    );
}

#[test]
fn ts_positive_v2_passes_with_multilang_resolver() {
    let (_keep, repo) = build_temp_repo("positive_v2_ts.md");
    let (output, _out_keep, json_path) = run_verb(&repo, "positive_v2_ts.md", true);
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap();
    assert!(
        output.status.success(),
        "v2 TS doc must pass --strict (decision={}, stderr={}): {}",
        report["decision"],
        String::from_utf8_lossy(&output.stderr),
        serde_json::to_string_pretty(&report).unwrap()
    );

    let claims = report["claims"].as_array().unwrap();
    let mismatches: usize = claims
        .iter()
        .filter(|c| c["note"].as_str() == Some("symbol mismatch"))
        .count();
    assert_eq!(
        mismatches,
        0,
        "v2 TS doc should produce zero symbol-mismatch findings: {}",
        serde_json::to_string_pretty(&claims).unwrap()
    );

    // All six TS claims should be present (4 methods + 1 top-level fn + 1 arrow binding).
    let verified: usize = claims
        .iter()
        .filter(|c| c["decision"].as_str() == Some("verified"))
        .count();
    assert!(
        verified >= 6,
        "expected ≥6 verified TS claims, got {}: {}",
        verified,
        serde_json::to_string_pretty(&claims).unwrap()
    );
}
