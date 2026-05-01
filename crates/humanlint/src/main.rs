use clap::Parser;
use humanlint::audit::run_audit;
use humanlint::render::{render_markdown, write_json, write_markdown};
use humanlint::versions::check_versions;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "humanlint", version)]
struct AuditArgs {
    #[arg(default_value = ".")]
    repo: PathBuf,
    #[arg(long, value_name = "PATH", default_value = "repo-score.json")]
    json: String,
    #[arg(long, value_name = "PATH", default_value = "repo-score.md")]
    md: String,
    #[arg(long)]
    changed: Vec<PathBuf>,
}

#[derive(Parser, Debug)]
#[command(name = "humanlint", version, disable_help_subcommand = true)]
struct VersionsArgs {
    #[arg(default_value = ".")]
    repo: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args_os();
    let _bin = args.next();
    match args.next() {
        Some(cmd) if cmd == "versions" => {
            let versions_args = VersionsArgs::parse_from(std::env::args_os().skip(2));
            check_versions(&versions_args.repo)?;
            return Ok(());
        }
        Some(cmd) if cmd == "audit" => {
            let args = AuditArgs::parse_from(std::env::args_os().skip(2));
            run_audit_and_write(args)?;
            return Ok(());
        }
        Some(_) | None => {}
    }

    let args = AuditArgs::parse();
    run_audit_and_write(args)?;
    Ok(())
}

fn run_audit_and_write(args: AuditArgs) -> anyhow::Result<()> {
    let report = run_audit(&args.repo, &args.changed)?;
    let json_text = serde_json::to_string_pretty(&report)?;
    let md_text = render_markdown(&report);
    write_json(&args.json, &json_text)?;
    write_markdown(&args.md, &md_text)?;
    eprintln!(
        "score={} raw={} caps={} findings={}",
        report.score,
        report.raw_score,
        report.caps_applied.len(),
        report.findings.len()
    );
    Ok(())
}
