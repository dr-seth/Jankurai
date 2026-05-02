use crate::audit::helpers::*;
use crate::model::DimensionResult;

pub fn analyze(ctx: &AuditContext) -> DimensionResult {
    let mut score = 20;
    let mut evidence = vec![];
    let mut notes = vec![];
    let surface_text = command_surface_text(ctx);
    if [
        "cargo check",
        "cargo nextest",
        "cargo build --timings",
        "sccache",
        "rust-cache",
        "bacon",
        "vitest",
        "pytest",
        "go test",
        "dotnet test",
        "pnpm",
        "bun test",
        "turbo",
        "nx",
        "tsc -p",
        "playwright test",
    ]
    .iter()
    .any(|n| surface_text.contains(n))
    {
        score += 20;
        evidence.push("build acceleration markers found".into());
    }
    if [
        "cargo check",
        "nextest",
        "vitest",
        "pytest",
        "go test",
        "dotnet test",
        "tsc -p",
        "playwright test",
    ]
    .iter()
    .any(|n| surface_text.contains(n))
    {
        score += 10;
        evidence.push("targeted test/build commands found".into());
    }
    if ctx.all_files.iter().any(|f| {
        [
            "Cargo.lock",
            "pnpm-lock.yaml",
            "package-lock.json",
            "yarn.lock",
            "uv.lock",
            "poetry.lock",
        ]
        .contains(&f.name.as_str())
    }) {
        score += 10;
        evidence.push("locked dependency graph present".into());
    }
    if ctx
        .all_files
        .iter()
        .any(|f| f.rel_path.starts_with(".github/workflows") && f.text.contains("cache"))
    {
        score += 10;
        evidence.push("CI cache hint found".into());
    }
    if !has_one_command(ctx) {
        score -= 10;
        notes.push("missing one-command setup/validation".into());
    }
    if !has_fast_lane(ctx) {
        score -= 10;
        notes.push("missing deterministic fast lane".into());
    }
    if real_command_surface_contains(ctx, &["cargo check"])
        && real_command_surface_contains(ctx, &["npm --workspace @jankurai/ux-qa run build"])
        && real_command_surface_contains(ctx, &["npm --workspace @jankurai/ux-qa run test"])
    {
        score += 15;
        evidence.push("focused Rust and UX QA build/test lanes are available".into());
    }
    make_dim("Build speed signals", score, evidence, notes)
}
