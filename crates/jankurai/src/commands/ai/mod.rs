//! `jankurai ai` command group. Slice 1 ships the single `audit` verb
//! (ARY-2030, verb #2 of the upstream contribution stream): scan a
//! directory tree for AI/LLM SDK call sites and classify each into one of
//! four replaceability tiers.

pub mod audit;

use anyhow::Result;
use std::path::PathBuf;

pub use audit::{AuditReport, SiteRecord};

/// Arguments for `jankurai ai audit <DIR...>`.
#[derive(Debug, Clone)]
pub struct AiAuditArgs {
    /// One or more directories to scan (recursively).
    pub dirs: Vec<PathBuf>,
    /// Optional `ai-audit.toml` config path; defaults applied when absent.
    pub config: Option<PathBuf>,
    /// Optional JSON report output path (stdout table is always printed).
    pub out: Option<String>,
    /// Optional markdown report output path.
    pub md: Option<String>,
}

/// Entry point dispatched from `Commands::Ai { AiCommand::Audit }`.
pub fn run(args: AiAuditArgs) -> Result<()> {
    audit::run(args)
}
