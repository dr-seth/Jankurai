// slice-risk standalone mode (ARY-2031) + postmortem record QO-schema
// validation & feedback loop (ARY-2032).
//
// Rule 9: env-blocker evidence is re-derived (filesystem stat of the
// declared checkpoint path; process-env + dotenv + keyfile probe for the
// secret) — never a string read of the TOML. Postmortem validation parses
// with a real TOML parser; a commented-out field is *absent*.
//
// Rule 8: the four `adversarial_*.toml` postmortems are the denial bench —
// `postmortem record` MUST reject each with the spec-mandated diagnostic.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_jankurai"))
}

fn fx(p: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(p)
}

/// Copy a fixture into a temp repo root so the repo-escape guard passes,
/// then run the verb from that root. Returns (exit_ok, stdout, stderr).
fn run_in_repo(
    fixture_rel: &str,
    extra_fixture: Option<&str>,
    args: &[&str],
) -> (bool, String, String) {
    let repo = tempdir().unwrap();
    // Mirror the whole fixtures/ tree the verb may touch.
    let dst = repo.path().join("fx");
    fs::create_dir_all(&dst).unwrap();
    for sub in ["slice-risk", "postmortem-record"] {
        let from = fx(sub);
        let to = dst.join(sub);
        copy_dir(&from, &to);
    }
    let mut cmd = Command::new(binary());
    cmd.arg("migrate").arg(repo.path());
    cmd.args(args);
    let _ = (fixture_rel, extra_fixture);
    let out = cmd.output().unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn copy_dir(from: &PathBuf, to: &PathBuf) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let p = entry.path();
        let dest = to.join(entry.file_name());
        if p.is_dir() {
            copy_dir(&p, &dest);
        } else {
            fs::copy(&p, &dest).unwrap();
        }
    }
}

#[test]
fn slice_risk_env_blocker_exits_nonzero_with_both_blockers() {
    let (ok, stdout, stderr) = run_in_repo(
        "",
        None,
        &[
            "slice-risk",
            "fx/slice-risk/positive_env_blocker_slice.toml",
        ],
    );
    assert!(!ok, "env-blocker slice must exit non-zero");
    let all = format!("{stdout}{stderr}");
    for needle in [
        "Environmental prerequisites",
        "models/synthetic_cascade.pt",
        "torch.load",
        "weights_only",
        "[BLOCKER]",
        "QORCH_TEAM_GATE_HMAC_KEY",
        "Slice cannot proceed until BLOCKERs are resolved",
        "Risk score:",
    ] {
        assert!(all.contains(needle), "missing `{needle}`:\n{all}");
    }
    // Re-derive the score band: must be 50..=100 for a blocked slice.
    let score = extract_score(&stdout).expect("Risk score line");
    assert!(
        (50..=100).contains(&score),
        "blocked score {score} not in 50..100"
    );
}

#[test]
fn slice_risk_mp_to_tokio_is_high_band_advisory() {
    let (ok, stdout, _e) = run_in_repo(
        "",
        None,
        &[
            "slice-risk",
            "fx/slice-risk/positive_mp_to_tokio_slice.toml",
        ],
    );
    assert!(ok, "mp->tokio is advisory (exit 0), not gating");
    for needle in [
        "Cross-runtime risk surface",
        "multiprocessing",
        "tokio",
        "HIGH risk pattern",
        "GIL re-entry",
    ] {
        assert!(stdout.contains(needle), "missing `{needle}`:\n{stdout}");
    }
    assert!(
        stdout.contains("numpy globals")
            || stdout.contains("OMP_NUM_THREADS")
            || stdout.contains("ESM cache"),
        "missing a named fork->thread global:\n{stdout}"
    );
    let score = extract_score(&stdout).expect("score");
    assert!(
        (50..=89).contains(&score),
        "HIGH band score {score} not in 50..89"
    );
}

#[test]
fn slice_risk_negative_control_is_low_band_no_blockers() {
    let (ok, stdout, _e) = run_in_repo(
        "",
        None,
        &[
            "slice-risk",
            "fx/slice-risk/negative_safety_kernel_slice.toml",
        ],
    );
    assert!(ok, "negative control must pass");
    assert!(
        !stdout.contains("[BLOCKER]"),
        "over-flagged a BLOCKER on the negative control:\n{stdout}"
    );
    assert!(
        !stdout.contains("HIGH risk pattern"),
        "over-flagged HIGH on the negative control:\n{stdout}"
    );
    let score = extract_score(&stdout).expect("score");
    assert!(
        score < 30,
        "negative control score {score} not in LOW band (<30)"
    );
}

#[test]
fn slice_risk_use_postmortems_emits_feedback_loop() {
    let (_ok, stdout, stderr) = run_in_repo(
        "",
        None,
        &[
            "slice-risk",
            "fx/slice-risk/positive_env_blocker_slice.toml",
            "--use-postmortems",
            "fx/postmortem-record/positive/atlas-2026-05.toml",
        ],
    );
    let all = format!("{stdout}{stderr}");
    for needle in [
        "Cross-referencing 1 prior postmortem",
        "2026-05-phase-3d-atlas",
        "applies here: torch.load checkpoint compat",
    ] {
        assert!(
            all.contains(needle),
            "feedback loop missing `{needle}`:\n{all}"
        );
    }
}

fn extract_score(text: &str) -> Option<u32> {
    let line = text.lines().find(|l| l.contains("Risk score:"))?;
    let after = line.split("Risk score:").nth(1)?;
    let num: String = after
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    num.parse().ok()
}

// --- Postmortem record (ARY-2032) ---

fn run_postmortem(repo: &std::path::Path, rel: &str) -> std::process::Output {
    Command::new(binary())
        .arg("postmortem")
        .arg(repo)
        .arg("record")
        .arg(rel)
        .output()
        .unwrap()
}

fn pm_repo() -> tempfile::TempDir {
    let repo = tempdir().unwrap();
    copy_dir(
        &fx("postmortem-record"),
        &repo.path().join("postmortem-record"),
    );
    repo
}

#[test]
fn postmortem_record_accepts_positive_and_round_trips() {
    let repo = pm_repo();
    let out = run_postmortem(repo.path(), "postmortem-record/positive/atlas-2026-05.toml");
    assert!(
        out.status.success(),
        "positive postmortem must be accepted: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("written to .jankurai/postmortems/2026-05-phase-3d-atlas.toml"),
        "missing derived-path line:\n{stdout}"
    );
    assert!(
        stdout.contains("canonical-hash:"),
        "missing hash line:\n{stdout}"
    );
    // Re-derive: the written file must exist and re-validate to the same hash.
    let written = repo
        .path()
        .join(".jankurai/postmortems/2026-05-phase-3d-atlas.toml");
    assert!(written.exists(), "record not written to derived path");
    let h1 = stdout
        .lines()
        .find(|l| l.contains("canonical-hash:"))
        .unwrap()
        .to_string();
    let out2 = run_postmortem(repo.path(), "postmortem-record/positive/atlas-2026-05.toml");
    let stdout2 = String::from_utf8_lossy(&out2.stdout);
    let h2 = stdout2
        .lines()
        .find(|l| l.contains("canonical-hash:"))
        .unwrap()
        .to_string();
    assert_eq!(h1, h2, "canonical hash not stable across runs");
}

#[test]
fn postmortem_record_rejects_four_adversarial_fixtures() {
    let cases: &[(&str, &[&str])] = &[
        (
            "adversarial_missing_failure_mode.toml",
            &["missing required field: failure_mode"],
        ),
        (
            "adversarial_invalid_failure_mode.toml",
            &[
                "invalid enum value for failure_mode",
                "totally-made-up-mode",
                "valid values:",
                "aspirational-spec, env-prerequisite, interop-runtime, equivalence-gap, cutover-rollback, perf-regression",
            ],
        ),
        (
            "adversarial_outcome_typo.toml",
            &["invalid enum value for outcome", "blocked-ish"],
        ),
        (
            "adversarial_missing_lessons.toml",
            &["missing required section: [lessons]"],
        ),
    ];
    let repo = pm_repo();
    for (file, needles) in cases {
        let out = run_postmortem(repo.path(), &format!("postmortem-record/{file}"));
        assert!(
            !out.status.success(),
            "{file} must be rejected (exit non-zero)"
        );
        let all = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        for n in *needles {
            assert!(all.contains(n), "{file}: missing diagnostic `{n}`:\n{all}");
        }
    }
}

#[test]
fn postmortem_record_commented_field_is_absent_not_present() {
    // Rule 9: `# failure_mode = "..."` (commented) MUST NOT satisfy the
    // required-field check. The missing_failure_mode fixture has exactly
    // that shape; assert it is rejected for the *field* reason.
    let repo = pm_repo();
    let out = run_postmortem(
        repo.path(),
        "postmortem-record/adversarial_missing_failure_mode.toml",
    );
    assert!(!out.status.success());
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        all.contains("missing required field: failure_mode"),
        "commented field wrongly treated as present:\n{all}"
    );
}
