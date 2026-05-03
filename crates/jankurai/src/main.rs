use clap::{Args, Parser, Subcommand};
use jankurai::audit::policy::AuditMode;
use jankurai::audit::{run_audit, run_audit_with_options, AuditOptions};
use jankurai::commands::{
    agent, bench, cell, certify, context_pack, doctor, govern, init, migrate, proof, registry,
    repair, repair_plan, security,
};
use jankurai::render::{render_markdown, write_json, write_markdown};
use jankurai::report::issues::IssueFormat;
use jankurai::validation::{self, ArtifactSchema};
use jankurai::versions::check_versions;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "jankurai", version, args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[command(flatten)]
    audit: AuditArgs,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Audit(AuditArgs),
    Init(InitArgs),
    Doctor(DoctorArgs),
    ContextPack(ContextPackArgs),
    RepairPlan(RepairPlanArgs),
    Lane(ProofPlanArgs),
    Proof(ProofPlanArgs),
    Prove(ProveArgs),
    Registry(RegistryArgs),
    Cell(CellArgs),
    Migrate(MigrateArgs),
    Bench(BenchArgs),
    Certify(CertifyArgs),
    Govern(GovernArgs),
    Repair(RepairArgs),
    Adapters {
        #[command(subcommand)]
        command: AdapterCommand,
    },
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    Ci {
        #[command(subcommand)]
        command: CiCommand,
    },
    Issues {
        #[command(subcommand)]
        command: IssuesCommand,
    },
    Explain(ExplainArgs),
    Versions(VersionsArgs),
    Ux(UxArgs),
    Security {
        #[command(subcommand)]
        command: SecurityCommand,
    },
}

#[derive(Subcommand, Debug)]
enum AdapterCommand {
    Verify(AdapterVerifyArgs),
    Sync(AdapterSyncArgs),
}

#[derive(Subcommand, Debug)]
enum CiCommand {
    Install(CiInstallArgs),
}

#[derive(Subcommand, Debug)]
enum IssuesCommand {
    Export(IssuesExportArgs),
}

#[derive(Subcommand, Debug)]
enum SecurityCommand {
    Run(SecurityRunArgs),
}

#[derive(Subcommand, Debug)]
enum AgentCommand {
    Verify(AgentVerifyArgs),
}

#[derive(Args, Debug, Clone)]
struct AuditArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "PATH", default_value = "agent/repo-score.json")]
    json: String,
    #[arg(long, value_name = "PATH", default_value = "agent/repo-score.md")]
    md: String,
    #[arg(long)]
    changed: Vec<PathBuf>,
    #[arg(long, value_name = "REF")]
    changed_from: Option<String>,
    #[arg(long, default_value = "standard")]
    mode: String,
    #[arg(long, value_name = "PATH")]
    sarif: Option<String>,
    #[arg(long, value_name = "PATH")]
    junit: Option<String>,
    #[arg(long, value_name = "PATH")]
    github_step_summary: Option<String>,
    #[arg(long, value_name = "PATH")]
    repair_queue_jsonl: Option<String>,
    #[arg(long, value_name = "PATH")]
    proof_receipts: Option<String>,
    #[arg(long, value_name = "PATH")]
    baseline: Option<String>,
    #[arg(long, value_name = "PATH")]
    policy: Option<String>,
    #[arg(long)]
    self_audit: bool,
    #[arg(long)]
    fail_under: Option<i32>,
    #[arg(long, value_delimiter = ',')]
    fail_on: Vec<String>,
}

#[derive(Args, Debug)]
struct InitArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long)]
    apply: bool,
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(long, default_value = "rust-ts-vite-react-postgres-bounded-python")]
    profile: String,
    #[arg(long, default_value = "all")]
    ide: String,
    #[arg(long, default_value = "advisory")]
    mode: String,
    #[arg(long)]
    diff: bool,
    #[arg(long, default_value = "github")]
    ci: String,
    #[arg(long, default_value = "jsonl")]
    issue_backend: String,
    #[arg(long)]
    ux_qa: bool,
    #[arg(long, value_name = "PATH")]
    plan_json: Option<String>,
    #[arg(long)]
    force_generated_adapters: bool,
}

#[derive(Args, Debug)]
struct DoctorArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, default_value = "high")]
    fail_on: String,
    #[arg(long, value_name = "PATH")]
    json: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
}

#[derive(Args, Debug)]
struct ContextPackArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long)]
    task: String,
    #[arg(long, value_name = "PATH")]
    changed: Vec<PathBuf>,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
}

#[derive(Args, Debug)]
struct RepairPlanArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "PATH")]
    from: String,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
}

#[derive(Args, Debug)]
struct ProofPlanArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long)]
    changed: Vec<PathBuf>,
    #[arg(long, value_name = "REF")]
    changed_from: Option<String>,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
}

#[derive(Args, Debug)]
struct ProveArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "PATH")]
    plan: Option<String>,
    #[arg(long)]
    changed: Vec<PathBuf>,
    #[arg(long, value_name = "REF")]
    changed_from: Option<String>,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/jankurai/proof-receipts"
    )]
    out_dir: String,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/jankurai/evidence-index.json"
    )]
    evidence_index: String,
    #[arg(long)]
    continue_on_error: bool,
    /// Allow commands not listed in agent/proof-lanes.toml or agent/test-map.json.
    /// Requires `JANKURAI_ALLOW_UNSIGNED_PROOF_COMMANDS=1` in the environment.
    #[arg(long = "allow-unsigned-commands")]
    allow_unsigned_commands: bool,
}

#[derive(Args, Debug)]
struct RegistryArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
}

#[derive(Args, Debug)]
struct CellArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, default_value = "workspace-cell")]
    cell_id: String,
    #[arg(long, default_value = "install-ready", value_parser = ["install-ready", "prove"])]
    mode: String,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
}

#[derive(Args, Debug)]
struct MigrateArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
    /// Run analyze mode (report only). Default is plan mode.
    #[arg(long)]
    analyze: bool,
}

#[derive(Args, Debug)]
struct BenchArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
}

#[derive(Args, Debug)]
struct CertifyArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
}

#[derive(Args, Debug)]
struct GovernArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
}

#[derive(Args, Debug)]
struct RepairArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "PATH")]
    plan: String,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    auto_pr: bool,
    #[arg(long, default_value = "low")]
    max_risk: String,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
}

#[derive(Args, Debug)]
struct VersionsArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
}

#[derive(Args, Debug)]
struct IssuesExportArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, default_value = "markdown")]
    format: String,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
}

#[derive(Args, Debug)]
struct CiInstallArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long)]
    github: bool,
    #[arg(long, default_value = "ratchet")]
    mode: String,
    #[arg(long, default_value_t = 85)]
    min_score: i32,
}

#[derive(Args, Debug)]
struct AdapterVerifyArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
}

#[derive(Args, Debug)]
struct AdapterSyncArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, default_value = "all")]
    ide: String,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    yes: bool,
}

#[derive(Args, Debug)]
struct AgentVerifyArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
}

#[derive(Args, Debug)]
struct ExplainArgs {
    rule_id: String,
}

#[derive(Args, Debug)]
struct UxArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<OsString>,
}

#[derive(Args, Debug)]
struct SecurityRunArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "PATH", default_value = "tools/security-lane.sh")]
    script: String,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/jankurai/security/evidence.json"
    )]
    out: String,
    #[arg(long)]
    strict: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Versions(args)) => {
            check_versions(&args.repo)?;
        }
        Some(Commands::Audit(args)) => {
            run_audit_and_write(args)?;
        }
        Some(Commands::Init(args)) => {
            init::run(init::InitArgs {
                repo: args.repo,
                apply: args.apply,
                dry_run: args.dry_run,
                yes: args.yes,
                profile: args.profile,
                ide: args.ide,
                mode: args.mode,
                diff: args.diff,
                ci: args.ci,
                issue_backend: args.issue_backend,
                ux_qa: args.ux_qa,
                plan_json: args.plan_json,
                force_generated_adapters: args.force_generated_adapters,
            })?;
        }
        Some(Commands::Doctor(args)) => {
            doctor::run(doctor::DoctorArgs {
                repo: args.repo,
                fail_on: args.fail_on,
                json: args.json,
                md: args.md,
            })?;
        }
        Some(Commands::ContextPack(args)) => {
            context_pack::run(context_pack::ContextPackArgs {
                repo: args.repo,
                task: args.task,
                changed: args.changed,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::RepairPlan(args)) => {
            repair_plan::run(repair_plan::RepairPlanArgs {
                repo: args.repo,
                from: args.from,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::Lane(args)) => {
            proof::run_lane(proof::ProofPlanArgs {
                repo: args.repo,
                changed: args.changed,
                changed_from: args.changed_from,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::Proof(args)) => {
            proof::run_proof(proof::ProofPlanArgs {
                repo: args.repo,
                changed: args.changed,
                changed_from: args.changed_from,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::Prove(args)) => {
            proof::run_prove(proof::ProveArgs {
                repo: args.repo,
                plan: args.plan,
                changed: args.changed,
                changed_from: args.changed_from,
                out_dir: args.out_dir,
                evidence_index: args.evidence_index,
                continue_on_error: args.continue_on_error,
                allow_unsigned_commands: args.allow_unsigned_commands,
            })?;
        }
        Some(Commands::Registry(args)) => {
            registry::run(registry::RegistryArgs {
                repo: args.repo,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::Cell(args)) => {
            cell::run(cell::CellArgs {
                repo: args.repo,
                cell_id: args.cell_id,
                mode: args.mode,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::Migrate(args)) => {
            let mode = if args.analyze {
                migrate::MigrateMode::Analyze
            } else {
                migrate::MigrateMode::Plan
            };
            migrate::run(migrate::MigrateArgs {
                repo: args.repo,
                out: args.out,
                md: args.md,
                mode,
            })?;
        }
        Some(Commands::Bench(args)) => {
            bench::run(bench::BenchArgs {
                repo: args.repo,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::Certify(args)) => {
            certify::run(certify::CertifyArgs {
                repo: args.repo,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::Govern(args)) => {
            govern::run(govern::GovernArgs {
                repo: args.repo,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::Repair(args)) => {
            repair::run(repair::RepairArgs {
                repo: args.repo,
                plan: args.plan,
                dry_run: args.dry_run,
                auto_pr: args.auto_pr,
                max_risk: args.max_risk,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::Adapters { command }) => match command {
            AdapterCommand::Verify(args) => run_adapters_verify(args)?,
            AdapterCommand::Sync(args) => run_adapters_sync(args)?,
        },
        Some(Commands::Agent { command }) => match command {
            AgentCommand::Verify(args) => {
                agent::verify(agent::AgentVerifyArgs { repo: args.repo })?
            }
        },
        Some(Commands::Issues { command }) => match command {
            IssuesCommand::Export(args) => run_issues_export(args)?,
        },
        Some(Commands::Ci { command }) => match command {
            CiCommand::Install(args) => {
                jankurai::commands::ci::install(jankurai::commands::ci::CiInstallArgs {
                    repo: args.repo,
                    github: args.github,
                    mode: args.mode,
                    min_score: args.min_score,
                })?;
            }
        },
        Some(Commands::Explain(args)) => run_explain(&args.rule_id)?,
        Some(Commands::Ux(_args)) => run_ux_passthrough()?,
        Some(Commands::Security { command }) => match command {
            SecurityCommand::Run(args) => {
                security::run(security::SecurityRunArgs {
                    repo: args.repo,
                    script: args.script,
                    out: args.out,
                    strict: args.strict,
                })?;
            }
        },
        None => {
            run_audit_and_write(cli.audit)?;
        }
    }
    Ok(())
}

fn parse_repo_arg(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);
    if path.exists()
        || value == "."
        || value == ".."
        || value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
        || value.contains(std::path::MAIN_SEPARATOR)
    {
        Ok(path)
    } else {
        Err(format!(
            "`{value}` is not a known command or an existing/path-like repository path"
        ))
    }
}

fn run_audit_and_write(args: AuditArgs) -> anyhow::Result<()> {
    if args.json == "-" && args.md == "-" {
        anyhow::bail!("use at most one stdout target; JSON and Markdown may not share stdout");
    }
    let changed = if let Some(base) = args.changed_from.as_deref() {
        jankurai::audit::changed_paths_from_git(&args.repo, base)?
    } else {
        args.changed
    };
    let mode = AuditMode::parse(&args.mode)?;
    let mut report = run_audit_with_options(
        &args.repo,
        &changed,
        AuditOptions {
            self_audit: args.self_audit,
            proof_receipts: args.proof_receipts.clone(),
        },
    )?;
    if let Some(minimum_score) = args.fail_under {
        if let Some(policy) = report.policy.as_mut() {
            policy.minimum_score = minimum_score;
        }
        if let Some(decision) = report.decision.as_mut() {
            decision.minimum_score = minimum_score;
            decision.passed = report.score >= minimum_score && decision.hard_findings == 0;
            decision.status = if decision.passed { "pass" } else { "fail" }.into();
        }
    }
    if !args.fail_on.is_empty() {
        if let Some(policy) = report.policy.as_mut() {
            policy.fail_on = args.fail_on.clone();
        }
    }
    apply_mode_and_baseline(&mut report, mode, args.baseline.as_deref())?;
    if matches!(mode, AuditMode::Release) && report.proof_receipts.is_empty() {
        report.findings.push(jankurai::model::Finding {
            severity: "high".into(),
            category: "proof".into(),
            path: "agent/test-map.json".into(),
            problem: "release mode requires proof receipts for the audited scope".into(),
            agent_fix: "run `jankurai prove` and feed its receipts into `jankurai audit --proof-receipts`".into(),
            evidence: vec!["no proof receipts were supplied".into()],
            check_id: "proof-receipts".into(),
            hardness: "hard".into(),
            confidence: 1.0,
            evidence_kind: "receipt".into(),
            rerun_command: "jankurai prove".into(),
            fingerprint: "sha256:pending".into(),
            rule_id: Some("HLT-004-UNMAPPED-PROOF".into()),
            tlr: Some("proof".into()),
            lane: Some("release".into()),
            docs_url: Some("agent/JANKURAI_STANDARD.md#proof-lanes".into()),
            owner: Some("agent".into()),
            line: None,
            matched_term: Some("proof receipts".into()),
            reason: Some("release mode cannot be verified without receipt evidence".into()),
        });
        jankurai::audit::rebuild_agent_fix_queue(&mut report);
        if let Some(decision) = report.decision.as_mut() {
            decision.hard_findings = report
                .findings
                .iter()
                .filter(|finding| matches!(finding.severity.as_str(), "high" | "critical"))
                .count();
            decision.soft_findings = report.findings.len().saturating_sub(decision.hard_findings);
            decision.passed = false;
            decision.status = "fail".into();
        }
    }
    report.report_fingerprint = jankurai::audit::report_fingerprint(&report);
    let md_text = render_markdown(&report);
    validation::write_json(&args.repo, ArtifactSchema::RepoScore, &args.json, &report)?;
    write_markdown(&args.md, &md_text)?;
    if let Some(path) = args.sarif.as_deref() {
        write_json(path, &jankurai::report::sarif::render_sarif(&report))?;
    }
    if let Some(path) = args.junit.as_deref() {
        write_json(path, &jankurai::report::junit::render_junit(&report))?;
    }
    if let Some(path) = args.github_step_summary.as_deref() {
        write_markdown(
            path,
            &jankurai::report::github::render_step_summary(&report),
        )?;
    }
    if let Some(path) = args.repair_queue_jsonl.as_deref() {
        write_json(
            path,
            &jankurai::report::issues::repair_queue_jsonl(&report),
        )?;
    }
    eprintln!(
        "score={} raw={} caps={} findings={}",
        report.score,
        report.raw_score,
        report.caps_applied.len(),
        report.findings.len()
    );
    Ok(())
}

fn run_adapters_verify(args: AdapterVerifyArgs) -> anyhow::Result<()> {
    let failures = jankurai::init::adapters::verify_adapters(&args.repo)?;
    if failures.is_empty() {
        println!("adapters verified");
        return Ok(());
    }
    for failure in failures {
        eprintln!("{}: {}", failure.path, failure.problem);
    }
    anyhow::bail!("adapter verification failed")
}

fn run_adapters_sync(args: AdapterSyncArgs) -> anyhow::Result<()> {
    if !args.dry_run && !args.yes {
        anyhow::bail!("refusing to write adapters without --dry-run or --yes");
    }
    let plan = jankurai::init::adapters::adapter_plan(&args.repo, &args.ide);
    for action in &plan {
        println!("{} {}", action.action, action.path);
    }
    if args.yes {
        jankurai::init::adapters::write_adapters(&args.repo, &args.ide, false)?;
    }
    Ok(())
}

fn run_explain(rule_id: &str) -> anyhow::Result<()> {
    let Some(doc) = jankurai::audit::docs_for_rule_id(rule_id) else {
        anyhow::bail!("unknown rule id `{rule_id}`");
    };
    println!("{rule_id}: {doc}");
    Ok(())
}

fn run_issues_export(args: IssuesExportArgs) -> anyhow::Result<()> {
    let format = IssueFormat::parse(&args.format)?;
    let report = run_audit(&args.repo, &[])?;
    let text = jankurai::report::issues::render_issues(&report, format);
    if let Some(out) = args.out.as_deref() {
        write_markdown(out, &text)?;
    } else {
        print!("{text}");
    }
    Ok(())
}

fn apply_mode_and_baseline(
    report: &mut jankurai::model::Report,
    mode: AuditMode,
    baseline: Option<&str>,
) -> anyhow::Result<()> {
    if let Some(policy) = report.policy.as_mut() {
        policy.mode = Some(mode.as_str().into());
    }
    if let Some(decision) = report.decision.as_mut() {
        if mode == AuditMode::Advisory {
            decision.status = "advisory".into();
            decision.passed = true;
        }
        if let Some(path) = baseline {
            let text = std::fs::read_to_string(path)?;
            let baseline_score = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|value| value.get("score").and_then(|score| score.as_i64()))
                .unwrap_or(report.score as i64) as i32;
            let ratchet_passed = report.score >= baseline_score;
            decision.ratchet = Some(jankurai::model::ReportRatchet {
                baseline_score,
                allowed_drop: 0,
                passed: ratchet_passed,
            });
            if matches!(mode, AuditMode::Ratchet | AuditMode::Release) && !ratchet_passed {
                decision.status = "fail".into();
                decision.passed = false;
            }
        }
    }
    Ok(())
}

fn run_ux_passthrough() -> anyhow::Result<()> {
    let status = Command::new("node")
        .arg("packages/ux-qa/dist/cli.js")
        .args(std::env::args_os().skip(2))
        .status()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("jankurai ux exited with {}", status)
    }
}
