pub struct Template {
    pub path: &'static str,
    pub body: &'static str,
}

pub fn template_for_path(path: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|t| t.path == path)
}

const ADAPTER_POINTER: &str = "<!-- jankurai generated adapter -->\nRead `AGENTS.md` first. Use `agent/JANKURAI_STANDARD.md` as the canonical jankurai standard.\nFor MASTER_PLAN work, read `agent/MASTER_PLAN.md`, then `tips/phases/00-phase-index.md`, then the active `tips/phases/*.md` phase file. Log phase work in `tips/phases/logs/`.\nFor planning work, follow `agent/MASTER_PLAN.md#detailed-planner-protocol`.\n";
const PROOF_ADAPTER_POINTER: &str = "# jankurai\n\n<!-- jankurai generated adapter -->\nRead `AGENTS.md` first. Use `agent/JANKURAI_STANDARD.md` as the canonical jankurai standard.\nFor MASTER_PLAN work, read `agent/MASTER_PLAN.md`, then `tips/phases/00-phase-index.md`, then the active `tips/phases/*.md` phase file. Log phase work in `tips/phases/logs/`.\nFor planning work, follow `agent/MASTER_PLAN.md#detailed-planner-protocol`.\nRun the proof lane in `agent/test-map.json` for changed paths.\n";

pub const TEMPLATES: &[Template] = &[
    Template {
        path: "AGENTS.md",
        body: "# Agent Instructions\n\nRead `agent/JANKURAI_STANDARD.md` first. For phase or MASTER_PLAN work, read `agent/MASTER_PLAN.md` before `tips/phases/00-phase-index.md`. Keep generated artifacts under their declared source commands.\n",
    },
    Template {
        path: ".cursor/rules/jankurai.mdc",
        body: "---\nalwaysApply: true\n---\n\n<!-- jankurai generated adapter -->\nRead `AGENTS.md` first. Use `agent/JANKURAI_STANDARD.md` as the canonical jankurai standard.\nFor MASTER_PLAN work, read `agent/MASTER_PLAN.md`, then `tips/phases/00-phase-index.md`, then the active `tips/phases/*.md` phase file. Log phase work in `tips/phases/logs/`.\nFor planning work, follow `agent/MASTER_PLAN.md#detailed-planner-protocol`.\n",
    },
    Template {
        path: "CLAUDE.md",
        body: ADAPTER_POINTER,
    },
    Template {
        path: "GEMINI.md",
        body: ADAPTER_POINTER,
    },
    Template {
        path: ".github/copilot-instructions.md",
        body: ADAPTER_POINTER,
    },
    Template {
        path: ".github/instructions/jankurai.instructions.md",
        body: ADAPTER_POINTER,
    },
    Template {
        path: ".github/instructions/jankurai-rust.instructions.md",
        body: "---\napplyTo: \"**/*.rs\"\n---\n\n<!-- jankurai generated adapter -->\nRead `AGENTS.md` first. Use `agent/JANKURAI_STANDARD.md` as the canonical jankurai standard.\nFor MASTER_PLAN work, read `agent/MASTER_PLAN.md`, then `tips/phases/00-phase-index.md`, then the active `tips/phases/*.md` phase file. Log phase work in `tips/phases/logs/`.\nFor planning work, follow `agent/MASTER_PLAN.md#detailed-planner-protocol`.\n",
    },
    Template {
        path: ".github/instructions/jankurai-web.instructions.md",
        body: "---\napplyTo: \"**/*.{ts,tsx,js,jsx,css}\"\n---\n\n<!-- jankurai generated adapter -->\nRead `AGENTS.md` first. Use `agent/JANKURAI_STANDARD.md` as the canonical jankurai standard.\nFor MASTER_PLAN work, read `agent/MASTER_PLAN.md`, then `tips/phases/00-phase-index.md`, then the active `tips/phases/*.md` phase file. Log phase work in `tips/phases/logs/`.\nFor planning work, follow `agent/MASTER_PLAN.md#detailed-planner-protocol`.\n",
    },
    Template {
        path: ".github/instructions/jankurai-python-ai.instructions.md",
        body: "---\napplyTo: \"python/ai-service/**/*.py\"\n---\n\n<!-- jankurai generated adapter -->\nRead `AGENTS.md` first. Use `agent/JANKURAI_STANDARD.md` as the canonical jankurai standard.\nFor MASTER_PLAN work, read `agent/MASTER_PLAN.md`, then `tips/phases/00-phase-index.md`, then the active `tips/phases/*.md` phase file. Log phase work in `tips/phases/logs/`.\nFor planning work, follow `agent/MASTER_PLAN.md#detailed-planner-protocol`.\n",
    },
    Template {
        path: ".agents/agents.md",
        body: ADAPTER_POINTER,
    },
    Template {
        path: ".agents/skills/jankurai/SKILL.md",
        body: PROOF_ADAPTER_POINTER,
    },
    Template {
        path: ".agents/workflows/jankurai-audit.md",
        body: "# jankurai audit\n\n<!-- jankurai generated adapter -->\nRead `AGENTS.md` first. Use `agent/JANKURAI_STANDARD.md` as the canonical jankurai standard.\nFor MASTER_PLAN work, read `agent/MASTER_PLAN.md`, then `tips/phases/00-phase-index.md`, then the active `tips/phases/*.md` phase file. Log phase work in `tips/phases/logs/`.\nFor planning work, follow `agent/MASTER_PLAN.md#detailed-planner-protocol`.\nRun `cargo run -p jankurai -- . --json agent/repo-score.json --md agent/repo-score.md` for audit.\n",
    },
    Template {
        path: ".claude/skills/jankurai/SKILL.md",
        body: PROOF_ADAPTER_POINTER,
    },
    Template {
        path: "agent/JANKURAI_STANDARD.md",
        body: "# jankurai Standard Agent Bootstrap\n\nStandard version: `0.4.0`\n\nRead `docs/agent-native-standard.md` when policy detail matters. Use `agent/owner-map.json`, `agent/test-map.json`, `agent/generated-zones.toml`, `agent/proof-lanes.toml`, and `agent/boundaries.toml` before editing.\n",
    },
    Template {
        path: "agent/MASTER_PLAN.md",
        body: "# jankurai Master Plan\n\nRead `agent/JANKURAI_STANDARD.md`, then this file, before phase or audit work.\n\nFor phase work, read `tips/phases/00-phase-index.md`, then the active `tips/phases/*.md` phase file. Pick the earliest incomplete phase whose dependencies can be advanced unless the user names a phase.\n\nWhen asked for planning, produce a worker-ready plan with objective, read-first files, ownership, current state, implementation steps, hard parts, validation, logging, and safe parallel work packets.\n\nAppend start, progress, and finish entries to `tips/phases/logs/<phase>.log`. Keep proof receipts and generated evidence under `target/jankurai/`.\n\nUse `agent/test-map.json` to choose the smallest credible proof lane. For audit, run `cargo run -p jankurai -- . --json agent/repo-score.json --md agent/repo-score.md`.\n",
    },
    Template {
        path: "agent/boundaries.toml",
        body: include_str!("../../templates/agent/boundaries.toml"),
    },
    Template {
        path: "agent/generated-zones.toml",
        body: include_str!("../../templates/agent/generated-zones.toml"),
    },
    Template {
        path: "agent/owner-map.json",
        body: include_str!("../../templates/agent/owner-map.json"),
    },
    Template {
        path: "agent/proof-lanes.toml",
        body: include_str!("../../templates/agent/proof-lanes.toml"),
    },
    Template {
        path: "agent/audit-policy.toml",
        body: "minimum_score = 85\nfail_on = [\"critical\", \"high\"]\nadvisory_on = [\"medium\", \"low\"]\n",
    },
    Template {
        path: "agent/security-policy.toml",
        body: "schema_version = \"1.0.0\"\nenabled_tools = [\"gitleaks\", \"cargo audit\", \"npm audit\"]\nrequired_tools = []\nadvisory_tools = [\"gitleaks\", \"cargo audit\", \"npm audit\"]\n\n[severity_thresholds]\nfail_lane_on = \"high\"\n",
    },
    Template {
        path: "agent/standard-version.toml",
        body: include_str!("../../templates/agent/standard-version.toml"),
    },
    Template {
        path: "agent/test-map.json",
        body: include_str!("../../templates/agent/test-map.json"),
    },
    Template {
        path: "docs/install.md",
        body: "# Install jankurai\n\nRun `jankurai init --profile rust-ts-postgres --ide all --mode advisory --dry-run`, review the plan, then rerun with `--yes`.\n",
    },
    Template {
        path: "docs/agent-native-standard.md",
        body: "# Agent-Native Standard\n\nKeep product truth in Rust, SQL, generated contracts, and bounded Python AI/data services. Route every path to an owner and proof lane.\n",
    },
    Template {
        path: "docs/ide-integrations.md",
        body: "# IDE Integrations\n\nAll IDE adapters are thin pointers to `agent/JANKURAI_STANDARD.md`; keep durable policy there or in `docs/`.\n",
    },
    Template {
        path: "docs/exceptions/README.md",
        body: "# jankurai Exceptions\n\nDocument dated exceptions with owner, expiry, migration path, and proof lane.\n",
    },
    Template {
        path: "README-jankurai-scaffold.md",
        body: "# Greenfield scaffold (non-production)\n\nThis tree was bootstrapped with `jankurai init --profile rust-ts-postgres`. Replace this file with a real product README when you have one.\n",
    },
    Template {
        path: "contracts/README.md",
        body: "# Contracts\n\nPut OpenAPI, JSON Schema, or protobuf **sources** here. Generated clients and bindings must live only under paths declared in `agent/generated-zones.toml`.\n",
    },
    Template {
        path: "db/README.md",
        body: "# Database\n\nMigrations live in `db/migrations/`. Optional constraint scripts in `db/constraints/`.\n",
    },
    Template {
        path: "db/migrations/README.md",
        body: "# Migrations\n\nAdd versioned SQL migrations. Regenerate any derived artifacts with the recorded command in `agent/generated-zones.toml`.\n",
    },
    Template {
        path: "db/constraints/README.md",
        body: "# Constraints\n\nDeclare durable database truth (checks, FKs) appropriate to your stack.\n",
    },
    Template {
        path: "docs/architecture/README.md",
        body: "# Architecture\n\nDocument boundaries, owners, proof lanes, and data flow. This stub is not production architecture.\n",
    },
    Template {
        path: "docs/decisions/README.md",
        body: "# Architecture Decision Records\n\nRecord significant decisions with date, status, context, and consequences.\n",
    },
    Template {
        path: "docs/auth/README.md",
        body: "# Authentication\n\nDocument authentication mechanisms, session lifecycles, and token issuance boundaries here.\n",
    },
    Template {
        path: "docs/orgs/README.md",
        body: "# Organizations\n\nDocument tenant isolation, RBAC, and cross-organization boundaries here.\n",
    },
    Template {
        path: "docs/admin/README.md",
        body: "# Admin Tools\n\nDocument elevated privileges, support masquerading, and internal operational routes here.\n",
    },
    Template {
        path: "tools/security-lane.sh",
        body: "#!/usr/bin/env bash\nset -euo pipefail\n# Scaffold stub: replace with real secret/dependency/SBOM checks (see jankurai `tools/security-lane.sh` in the standard repo).\necho \"security-lane scaffold: ok\"\n",
    },
    Template {
        path: "agent/ux-qa.toml",
        body: "outputRoot = \".\"\nartifactRoot = \"target/jankurai/ux-qa\"\nreadyState = \"domcontentloaded\"\ntimeoutMs = 15000\nscreenshotRequired = true\nariaSnapshotRequired = true\naccessibilityScanRequired = true\nrequiredStates = [\"loading\", \"empty\", \"error\", \"success\", \"permission-denied\"]\n",
    },
    Template {
        path: ".github/workflows/jankurai.yml",
        body: "name: jankurai\n\non:\n  pull_request:\n  push:\n    branches: [main]\n\njobs:\n  audit:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: dtolnay/rust-toolchain@stable\n      - uses: actions/setup-node@v4\n        with:\n          node-version: \"22\"\n          cache: npm\n      - run: npm ci\n      - run: cargo run -p jankurai -- versions\n      - name: jankurai audit\n        run: cargo run -p jankurai -- audit . --mode ratchet --json agent/repo-score.json --md agent/repo-score.md --sarif target/jankurai/jankurai.sarif --github-step-summary target/jankurai/summary.md --repair-queue-jsonl target/jankurai/repair-queue.jsonl\n      - name: Enforce score floor\n        run: test \"$(jq -r '.score' agent/repo-score.json)\" -ge 85\n      - name: Security lane\n        run: bash tools/security-lane.sh\n      - uses: actions/upload-artifact@v4\n        with:\n          name: jankurai-score\n          path: |\n            agent/repo-score.json\n            agent/repo-score.md\n            target/jankurai/jankurai.sarif\n            target/jankurai/repair-queue.jsonl\n",
    },
];
