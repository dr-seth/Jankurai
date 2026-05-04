use clap::{Args, Parser, Subcommand};
use jankurai::audit::policy::AuditMode;
use jankurai::audit::{run_audit, run_audit_with_options, AuditOptions};
use jankurai::commands::{
    adopt, agent, bench, cell, certify, context_pack, doctor, exceptions, govern, hooks, init,
    migrate, optimize, proof, publish, registry, repair, repair_plan, rust, security, update,
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
    Adopt(AdoptArgs),
    Init(InitArgs),
    Update(UpdateArgs),
    Doctor(DoctorArgs),
    ContextPack(ContextPackArgs),
    RepairPlan(RepairPlanArgs),
    Lane(ProofPlanArgs),
    Proof(ProofPlanArgs),
    Prove(ProveArgs),
    ProofVerify(ProofVerifyArgs),
    Registry(RegistryArgs),
    Cell(CellArgs),
    Migrate(MigrateArgs),
    Bench(BenchArgs),
    Certify(CertifyArgs),
    Govern(GovernArgs),
    Publish(PublishArgs),
    Repair(RepairArgs),
    Optimize(OptimizeArgs),
    Rust {
        #[command(subcommand)]
        command: RustCommand,
    },
    Exceptions {
        #[command(subcommand)]
        command: ExceptionCommand,
    },
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
    Hooks {
        #[command(subcommand)]
        command: HooksCommand,
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
enum HooksCommand {
    Install(HooksInstallArgs),
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
enum RustCommand {
    Map(RustMapArgs),
    Witness {
        #[command(subcommand)]
        command: RustWitnessCommand,
    },
    Diagnose(RustDiagnoseArgs),
}

#[derive(Subcommand, Debug)]
enum RustWitnessCommand {
    Build(RustWitnessBuildArgs),
    Diff(RustWitnessDiffArgs),
}

#[derive(Subcommand, Debug)]
enum ExceptionCommand {
    Expire(ExceptionExpireArgs),
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
    proof_evidence: Option<String>,
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
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/jankurai/score-history.jsonl"
    )]
    score_history: String,
    #[arg(long, value_name = "PATH")]
    score_history_csv: Option<String>,
    #[arg(long)]
    no_score_history: bool,
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
    /// Init profile manifest JSON (`schemas/init-profile.schema.json`). When set, bundled `--profile` is not used to resolve the manifest.
    #[arg(long, value_name = "PATH")]
    profile_file: Option<PathBuf>,
    #[arg(long, default_value = "full", value_parser = ["agents", "score", "ci", "full"])]
    level: String,
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
    /// Full-send adoption: apply the full scaffold, install observe CI, score, append tracked history, stage everything, and commit.
    #[arg(long)]
    yolo: bool,
    #[arg(long, default_value = "Adopt Jankurai control plane")]
    yolo_message: String,
}

#[derive(Args, Debug)]
struct UpdateArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long)]
    check: bool,
    #[arg(long)]
    apply: bool,
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    self_update: bool,
    #[arg(long, hide = true)]
    skip_self: bool,
    #[arg(long)]
    client_start: bool,
    #[arg(long)]
    quiet: bool,
    #[arg(long, default_value = "stable", value_parser = ["stable", "beta", "draft", "lts"])]
    channel: String,
    #[arg(long, default_value = "auto", value_parser = ["auto", "crates-io", "git", "local"])]
    source: String,
    #[arg(long)]
    offline: bool,
    #[arg(long)]
    fail_if_outdated: bool,
    #[arg(long)]
    install_missing: bool,
    #[arg(long, default_value = "rust-ts-postgres")]
    profile: String,
    #[arg(long, default_value = "full", value_parser = ["agents", "score", "ci", "full"])]
    level: String,
    #[arg(long, default_value = "all")]
    ide: String,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/jankurai/update/update-plan.json"
    )]
    out: String,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/jankurai/update/update-plan.md"
    )]
    md: String,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/jankurai/update/state.json"
    )]
    state: String,
}

#[derive(Args, Debug)]
struct AdoptArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, default_value = "auto", value_parser = [
        "auto",
        "migration-target",
        "rust-api",
        "react-web",
        "rust-ts-postgres",
        "b2b-saas",
        "ai-product",
        "regulated-saas"
    ])]
    profile: String,
    #[arg(long, default_value = "observe", value_parser = ["observe", "advisory", "ratchet"])]
    mode: String,
    #[arg(long, value_name = "PATH", default_value = adopt::DEFAULT_OUT)]
    out: String,
    #[arg(long, value_name = "PATH", default_value = adopt::DEFAULT_MD)]
    md: String,
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
        default_value = "target/jankurai/proof-plan.json"
    )]
    plan_out: String,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/jankurai/proof-plan.md"
    )]
    plan_md: String,
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
struct ProofVerifyArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "PATH")]
    plan: String,
    #[arg(long, value_name = "PATH")]
    evidence_index: String,
    #[arg(long, value_name = "PATH")]
    out: String,
    #[arg(long, value_name = "PATH")]
    md: String,
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
    #[arg(long, default_value = "install-ready", value_parser = ["install-ready", "prove", "upgrade-plan", "deprecate-plan"])]
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
    /// Target stack for migration (default: rust-ts-postgres)
    #[arg(long, default_value = "rust-ts-postgres")]
    target: String,
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
struct PublishArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/jankurai/p12-certification.json"
    )]
    certification: String,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/jankurai/p12-benchmark-report.json"
    )]
    benchmark: String,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/jankurai/p12-governance-policy.json"
    )]
    governance: String,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
    #[arg(long, value_name = "PATH")]
    badge_json: Option<String>,
    #[arg(long, value_name = "PATH")]
    badge_svg: Option<String>,
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
    fixture_apply: bool,
    /// Apply a bounded repair to a real git repository. Requires
    /// JANKURAI_ALLOW_REPAIR_APPLY=1 in the environment.
    #[arg(long)]
    apply: bool,
    #[arg(long)]
    auto_pr: bool,
    /// With --apply, commit repair changes and optionally push. Requires
    /// JANKURAI_ALLOW_GIT_MUTATION=1 in the environment.
    #[arg(long)]
    git_commit: bool,
    /// With --apply --git-commit --auto-pr, push and create a draft GitHub PR
    /// via gh. Requires JANKURAI_ALLOW_GITHUB_PR=1 in the environment.
    #[arg(long)]
    github_pr: bool,
    #[arg(long, default_value = "origin")]
    remote: String,
    #[arg(long, default_value = "main")]
    base: String,
    #[arg(long, value_name = "PATH")]
    pr_draft_out: Option<String>,
    #[arg(long, value_name = "PATH")]
    pr_draft_md: Option<String>,
    #[arg(long, default_value = "low")]
    max_risk: String,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
}

#[derive(Args, Debug)]
struct OptimizeArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, default_value = "all", value_parser = ["all", "token", "performance", "dependency", "dead-code"])]
    mode: String,
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    #[arg(long, value_name = "PATH")]
    md: Option<String>,
}

#[derive(Args, Debug)]
struct ExceptionExpireArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, default_value_t = 7)]
    warning_days: i64,
    /// Exit with failure when the report status is blocked (expired or invalid exceptions). Expiring-soon remains status complete.
    #[arg(long)]
    strict: bool,
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
    #[arg(long, default_value = "observe", value_parser = ["observe", "advisory", "ratchet"])]
    mode: String,
    #[arg(long, default_value_t = 85)]
    min_score: i32,
    #[arg(long, value_name = "PATH")]
    baseline: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

#[derive(Args, Debug)]
struct HooksInstallArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    force: bool,
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

#[derive(Args, Debug)]
struct RustMapArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, default_value = "target/jankurai/rust")]
    out_dir: String,
}

#[derive(Args, Debug)]
struct RustWitnessBuildArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, default_value = "target/jankurai/rust/witness-graph.json")]
    out: String,
}

#[derive(Args, Debug)]
struct RustWitnessDiffArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "FILE")]
    old: PathBuf,
    #[arg(long, value_name = "FILE")]
    new: PathBuf,
    #[arg(long, default_value = "target/jankurai/rust/witness-diff.json")]
    out: String,
    #[arg(long, default_value = "target/jankurai/rust/witness-diff.md")]
    md: String,
}

#[derive(Args, Debug)]
struct RustDiagnoseArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, default_value = "target/jankurai/rust/compile-packets.json")]
    out: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse_from(normalize_cli_args(std::env::args_os()));
    match cli.command {
        Some(Commands::Versions(args)) => {
            check_versions(&args.repo)?;
        }
        Some(Commands::Audit(args)) => {
            run_audit_and_write(args)?;
        }
        Some(Commands::Adopt(args)) => {
            adopt::run(adopt::AdoptArgs {
                repo: args.repo,
                profile: args.profile,
                mode: args.mode,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::Init(args)) => {
            if args.yolo {
                run_init_yolo(args)?;
                return Ok(());
            }
            init::run(init::InitArgs {
                repo: args.repo,
                apply: args.apply,
                dry_run: args.dry_run,
                yes: args.yes,
                profile: args.profile,
                profile_file: args.profile_file,
                level: args.level,
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
        Some(Commands::Update(args)) => {
            update::run(update::UpdateArgs {
                repo: args.repo,
                check: args.check,
                apply: args.apply,
                yes: args.yes,
                self_update: args.self_update,
                skip_self: args.skip_self,
                client_start: args.client_start,
                quiet: args.quiet,
                channel: args.channel,
                source: args.source,
                offline: args.offline,
                fail_if_outdated: args.fail_if_outdated,
                install_missing: args.install_missing,
                profile: args.profile,
                level: args.level,
                ide: args.ide,
                out: args.out,
                md: args.md,
                state: args.state,
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
                plan_out: args.plan_out,
                plan_md: args.plan_md,
                out_dir: args.out_dir,
                evidence_index: args.evidence_index,
                continue_on_error: args.continue_on_error,
                allow_unsigned_commands: args.allow_unsigned_commands,
            })?;
        }
        Some(Commands::ProofVerify(args)) => {
            proof::run_proof_verify(proof::ProofVerifyArgs {
                repo: args.repo,
                plan: args.plan,
                evidence_index: args.evidence_index,
                out: args.out,
                md: args.md,
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
                target: args.target,
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
        Some(Commands::Publish(args)) => {
            publish::run(publish::PublishArgs {
                repo: args.repo,
                certification: args.certification,
                benchmark: args.benchmark,
                governance: args.governance,
                out: args.out,
                md: args.md,
                badge_json: args.badge_json,
                badge_svg: args.badge_svg,
            })?;
        }
        Some(Commands::Repair(args)) => {
            repair::run(repair::RepairArgs {
                repo: args.repo,
                plan: args.plan,
                dry_run: args.dry_run,
                fixture_apply: args.fixture_apply,
                apply: args.apply,
                auto_pr: args.auto_pr,
                git_commit: args.git_commit,
                github_pr: args.github_pr,
                remote: args.remote,
                base: args.base,
                pr_draft_out: args.pr_draft_out,
                pr_draft_md: args.pr_draft_md,
                max_risk: args.max_risk,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::Optimize(args)) => {
            optimize::run(optimize::OptimizeArgs {
                repo: args.repo,
                mode: args.mode,
                out: args.out,
                md: args.md,
            })?;
        }
        Some(Commands::Rust { command }) => match command {
            RustCommand::Map(args) => {
                rust::run_map(rust::RustMapArgs {
                    repo: args.repo,
                    out_dir: args.out_dir,
                })?;
            }
            RustCommand::Witness { command } => match command {
                RustWitnessCommand::Build(args) => {
                    rust::run_witness_build(rust::RustWitnessBuildArgs {
                        repo: args.repo,
                        out: args.out,
                    })?;
                }
                RustWitnessCommand::Diff(args) => {
                    rust::run_witness_diff(rust::RustWitnessDiffArgs {
                        repo: args.repo,
                        old: args.old,
                        new: args.new,
                        out: args.out,
                        md: args.md,
                    })?;
                }
            },
            RustCommand::Diagnose(args) => {
                rust::run_diagnose(rust::RustDiagnoseArgs {
                    repo: args.repo,
                    out: args.out,
                })?;
            }
        },
        Some(Commands::Exceptions { command }) => match command {
            ExceptionCommand::Expire(args) => {
                exceptions::run_expire(exceptions::ExceptionExpireArgs {
                    repo: args.repo,
                    warning_days: args.warning_days,
                    strict: args.strict,
                    out: args.out,
                    md: args.md,
                })?;
            }
        },
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
                    baseline: args.baseline,
                    dry_run: args.dry_run,
                })?;
            }
        },
        Some(Commands::Hooks { command }) => match command {
            HooksCommand::Install(args) => hooks::install(hooks::HooksInstallArgs {
                repo: args.repo,
                yes: args.yes,
                dry_run: args.dry_run,
                force: args.force,
            })?,
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

fn normalize_cli_args(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut args: Vec<OsString> = args.into_iter().collect();
    if args.len() < 2 {
        return args;
    }
    let alias = args[1].to_string_lossy();
    if alias != "--update" && alias != "-update" {
        return args;
    }
    let mut normalized = vec![args.remove(0), OsString::from("update")];
    if args.len() == 1 || args[1].to_string_lossy().starts_with('-') {
        normalized.push(OsString::from("."));
    }
    normalized.extend(args.into_iter().skip(1));
    normalized
}

fn run_init_yolo(args: InitArgs) -> anyhow::Result<()> {
    if args.dry_run || args.diff {
        anyhow::bail!("--yolo commits changes; omit --dry-run/--diff or run normal init first");
    }
    if !args.yes {
        eprintln!(
            "{}",
            jankurai::ui::epaint(
                jankurai::ui::Style::Warn,
                "--yolo implies --yes, --level full, observe CI, local hooks, score history, score trailers, git add -A, and git commit"
            )
        );
    }

    ensure_git_repo(&args.repo)?;
    let repo = args.repo.clone();
    let adoption_json = repo.join("agent/adoption-plan.json");
    let adoption_md = repo.join("agent/adoption-plan.md");
    let score_json = repo.join("agent/repo-score.json");
    let score_md = repo.join("agent/repo-score.md");
    let history_jsonl = repo.join("agent/score-history.jsonl");
    let history_csv = repo.join("agent/score-history.csv");
    let doctor_json = repo.join("target/jankurai/doctor.json");
    let doctor_md = repo.join("target/jankurai/doctor.md");

    adopt::run(adopt::AdoptArgs {
        repo: repo.clone(),
        profile: args.profile.clone(),
        mode: "observe".into(),
        out: adoption_json.display().to_string(),
        md: adoption_md.display().to_string(),
    })?;

    init::run(init::InitArgs {
        repo: repo.clone(),
        apply: false,
        dry_run: false,
        yes: true,
        profile: args.profile,
        profile_file: args.profile_file,
        level: "full".into(),
        ide: args.ide,
        mode: args.mode,
        diff: false,
        ci: args.ci,
        issue_backend: args.issue_backend,
        ux_qa: args.ux_qa,
        plan_json: args.plan_json,
        force_generated_adapters: args.force_generated_adapters,
    })?;

    jankurai::commands::ci::install(jankurai::commands::ci::CiInstallArgs {
        repo: repo.clone(),
        github: true,
        mode: "observe".into(),
        min_score: 85,
        baseline: None,
        dry_run: false,
    })?;

    hooks::install(hooks::HooksInstallArgs {
        repo: repo.clone(),
        yes: true,
        dry_run: false,
        force: false,
    })?;

    run_audit_and_write(AuditArgs {
        repo: repo.clone(),
        json: score_json.display().to_string(),
        md: score_md.display().to_string(),
        changed: vec![],
        changed_from: None,
        mode: "advisory".into(),
        sarif: None,
        junit: None,
        github_step_summary: None,
        repair_queue_jsonl: None,
        proof_receipts: None,
        proof_evidence: None,
        baseline: None,
        policy: None,
        self_audit: false,
        fail_under: None,
        fail_on: vec![],
        score_history: history_jsonl.display().to_string(),
        score_history_csv: Some(history_csv.display().to_string()),
        no_score_history: false,
    })?;
    let score_trailers = score_trailers_from_report(&repo, &score_json)?;

    doctor::run(doctor::DoctorArgs {
        repo: repo.clone(),
        fail_on: "critical".into(),
        json: Some(doctor_json.display().to_string()),
        md: Some(doctor_md.display().to_string()),
    })?;

    run_git(&repo, &["add", "-A"])?;
    let staged = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(&repo)
        .status()?;
    if staged.success() {
        println!("--yolo found no staged changes to commit");
        return Ok(());
    }
    run_git_env(
        &repo,
        &["commit", "-m", &args.yolo_message, "-m", &score_trailers],
        &[("JANKURAI_SKIP_HOOKS", "1")],
    )?;
    if let Some(commit) = git_stdout(&repo, &["rev-parse", "--short", "HEAD"])? {
        println!(
            "{}",
            jankurai::ui::paint(
                jankurai::ui::Style::Good,
                format!("--yolo committed {commit}"),
                jankurai::ui::stdout_color_enabled()
            )
        );
    }
    print_yolo_next_steps(&repo);
    Ok(())
}

fn print_yolo_next_steps(repo: &std::path::Path) {
    let color = jankurai::ui::stdout_color_enabled();
    println!(
        "{}",
        jankurai::ui::paint(jankurai::ui::Style::Heading, "YOLO complete. Next:", color)
    );
    println!("  1. Push the adoption commit when ready: `git push -u origin HEAD`.");
    println!(
        "  2. Start Codex, OpenCode, Claude, Cursor, or another agent from `{}`.",
        repo.display()
    );
    println!(
        "  3. Tell it: `{}`",
        jankurai::ui::paint(
            jankurai::ui::Style::Accent,
            "Read AGENTS.md, follow the jankurai standard, improve the score, and commit small steps. Local hooks now auto-score each commit.",
            color
        )
    );
    println!(
        "- score history: `{}` and `{}`",
        repo.join("agent/score-history.jsonl").display(),
        repo.join("agent/score-history.csv").display()
    );
}

fn ensure_git_repo(repo: &std::path::Path) -> anyhow::Result<()> {
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(repo)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("--yolo requires an existing git repository");
    }
    Ok(())
}

fn run_git(repo: &std::path::Path, args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new("git").args(args).current_dir(repo).status()?;
    if !status.success() {
        anyhow::bail!("git {} failed with status {}", args.join(" "), status);
    }
    Ok(())
}

fn run_git_env(repo: &std::path::Path, args: &[&str], envs: &[(&str, &str)]) -> anyhow::Result<()> {
    let mut command = Command::new("git");
    command.args(args).current_dir(repo);
    for (key, value) in envs {
        command.env(key, value);
    }
    let status = command.status()?;
    if !status.success() {
        anyhow::bail!("git {} failed with status {}", args.join(" "), status);
    }
    Ok(())
}

fn git_stdout(repo: &std::path::Path, args: &[&str]) -> anyhow::Result<Option<String>> {
    let output = Command::new("git").args(args).current_dir(repo).output()?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&output.stdout).trim().to_string()).filter(|s| !s.is_empty()))
}

fn score_trailers_from_report(
    repo: &std::path::Path,
    score_json: &std::path::Path,
) -> anyhow::Result<String> {
    let text = std::fs::read_to_string(score_json)?;
    let value: serde_json::Value = serde_json::from_str(&text)?;
    let score = value.get("score").and_then(|v| v.as_i64()).unwrap_or(0);
    let raw_score = value
        .get("raw_score")
        .and_then(|v| v.as_i64())
        .unwrap_or(score);
    let finding_count = value
        .get("findings")
        .and_then(|v| v.as_array())
        .map(|v| v.len())
        .unwrap_or(0);
    let decision = value.get("decision");
    let hard_findings = decision
        .and_then(|v| v.get("hard_findings"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let minimum_score = decision
        .and_then(|v| v.get("minimum_score"))
        .and_then(|v| v.as_i64())
        .unwrap_or(85);
    let status = if hard_findings > 0 || score < minimum_score {
        "fail"
    } else {
        "pass"
    };
    let report = score_json
        .strip_prefix(repo)
        .unwrap_or(score_json)
        .to_string_lossy()
        .replace('\\', "/");
    Ok(format!(
        "Jankurai-Score: {score}\nJankurai-Raw-Score: {raw_score}\nJankurai-Findings: {finding_count}\nJankurai-Hard-Findings: {hard_findings}\nJankurai-Decision: {status}\nJankurai-Report: {report}"
    ))
}

fn run_audit_and_write(args: AuditArgs) -> anyhow::Result<()> {
    if args.json == "-" && args.md == "-" {
        anyhow::bail!("use at most one stdout target; JSON and Markdown may not share stdout");
    }
    let progress = jankurai::ui::CliProgress::new("scoring repository", 8);
    progress.tick("resolve changed paths");
    let changed = if let Some(base) = args.changed_from.as_deref() {
        jankurai::audit::changed_paths_from_git(&args.repo, base)?
    } else {
        args.changed
    };
    progress.tick("load audit mode");
    let mode = AuditMode::parse(&args.mode)?;
    progress.tick("scan repository");
    let mut report = run_audit_with_options(
        &args.repo,
        &changed,
        AuditOptions {
            self_audit: args.self_audit,
            proof_receipts: args.proof_receipts.clone(),
        },
    )?;
    progress.tick("apply score policy");
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
    progress.tick("apply mode and baseline");
    apply_mode_and_baseline(&mut report, mode, args.baseline.as_deref())?;
    if matches!(mode, AuditMode::Release) {
        let proof_findings = jankurai::audit::release_proof_findings(
            &args.repo,
            args.proof_receipts.as_deref(),
            args.proof_evidence.as_deref(),
        )?;
        if !proof_findings.is_empty() {
            report.findings.extend(proof_findings);
            jankurai::audit::rebuild_agent_fix_queue(&mut report);
            if let Some(decision) = report.decision.as_mut() {
                decision.hard_findings = report
                    .findings
                    .iter()
                    .filter(|finding| matches!(finding.severity.as_str(), "high" | "critical"))
                    .count();
                decision.soft_findings =
                    report.findings.len().saturating_sub(decision.hard_findings);
                decision.passed = false;
                decision.status = "fail".into();
            }
        }
    }
    progress.tick("render artifacts");
    report.report_fingerprint = jankurai::audit::report_fingerprint(&report);
    let md_text = render_markdown(&report);
    progress.tick("write JSON and Markdown");
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
        write_json(path, &jankurai::report::issues::repair_queue_jsonl(&report))?;
    }
    if !args.no_score_history {
        let history_path = jankurai::score_history::append_score_history(
            &args.repo,
            &report,
            &args.json,
            &args.md,
            &args.score_history,
            args.score_history_csv.as_deref(),
        )?;
        eprintln!("score history appended {}", history_path.display());
    }
    progress.finish(format!(
        "score {} raw {} findings {}",
        report.score,
        report.raw_score,
        report.findings.len()
    ));
    eprintln!(
        "{}",
        jankurai::ui::epaint(
            jankurai::ui::Style::Good,
            format!(
                "score={} raw={} caps={} findings={}",
                report.score,
                report.raw_score,
                report.caps_applied.len(),
                report.findings.len()
            )
        )
    );
    Ok(())
}

fn run_adapters_verify(args: AdapterVerifyArgs) -> anyhow::Result<()> {
    let failures = jankurai::init::adapters::verify_adapters(&args.repo)?;
    if failures.is_empty() {
        println!(
            "{}",
            jankurai::ui::paint(
                jankurai::ui::Style::Good,
                "adapters verified",
                jankurai::ui::stdout_color_enabled()
            )
        );
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
