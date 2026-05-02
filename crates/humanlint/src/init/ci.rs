pub fn audit_command(mode: &str) -> String {
    format!(
        "cargo run -p humanlint -- audit . --mode {mode} --json agent/repo-score.json --md agent/repo-score.md"
    )
}
