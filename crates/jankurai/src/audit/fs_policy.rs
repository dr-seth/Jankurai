use crate::model::FileInfo;
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::path::Path;

const DEFAULT_MAX_CAPTURE_CHARS: usize = 120_000;

#[derive(Debug, Clone)]
pub struct InventoryOptions {
    pub text_capture_chars: usize,
    pub extra_excluded_paths: Vec<String>,
    pub extra_excluded_globs: Option<GlobSet>,
}

impl Default for InventoryOptions {
    fn default() -> Self {
        Self {
            text_capture_chars: DEFAULT_MAX_CAPTURE_CHARS,
            extra_excluded_paths: vec![],
            extra_excluded_globs: None,
        }
    }
}

impl InventoryOptions {
    pub fn from_policy(root: &Path) -> Self {
        #[derive(Debug, Deserialize, Default)]
        struct AuditPolicyFile {
            #[serde(default)]
            scan: ScanPolicy,
        }

        #[derive(Debug, Deserialize, Default)]
        struct ScanPolicy {
            text_capture_chars: Option<usize>,
            max_capture_chars: Option<usize>,
            #[serde(default)]
            extra_excluded_paths: Vec<String>,
            #[serde(default)]
            extra_excluded_globs: Vec<String>,
        }

        let parsed = std::fs::read_to_string(root.join("agent/audit-policy.toml"))
            .ok()
            .and_then(|text| toml::from_str::<AuditPolicyFile>(&text).ok())
            .unwrap_or_default();
        let scan = parsed.scan;
        let mut options = Self {
            text_capture_chars: scan
                .text_capture_chars
                .or(scan.max_capture_chars)
                .unwrap_or(DEFAULT_MAX_CAPTURE_CHARS),
            extra_excluded_paths: scan
                .extra_excluded_paths
                .into_iter()
                .map(|path| {
                    path.trim()
                        .trim_start_matches("./")
                        .trim_end_matches('/')
                        .to_string()
                })
                .filter(|path| !path.is_empty())
                .collect(),
            extra_excluded_globs: build_globset(&scan.extra_excluded_globs),
        };
        if options.text_capture_chars == 0 {
            options.text_capture_chars = DEFAULT_MAX_CAPTURE_CHARS;
        }
        options
    }
}

fn build_globset(globs: &[String]) -> Option<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    let mut added = false;
    for glob in globs {
        let glob = glob.trim();
        if glob.is_empty() {
            continue;
        }
        if let Ok(glob) = Glob::new(glob) {
            builder.add(glob);
            added = true;
        }
    }
    if added {
        builder.build().ok()
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct InventoryResult {
    pub files: Vec<FileInfo>,
    pub timings: InventoryTimings,
}

#[derive(Debug, Clone, Default)]
pub struct InventoryTimings {
    pub walk_ms: u128,
    pub metadata_ms: u128,
    pub text_capture_ms: u128,
}
