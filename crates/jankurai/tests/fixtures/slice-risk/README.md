# Verb #3 — `jankurai migrate slice-risk <slice>` fixtures

Verb: static-analyze a migration slice plan + its target directory, surface env-prerequisite blockers and cross-runtime risk patterns. Risk score is advisory, blockers are gating.

**Current shape (v0)**: each fixture is a `slice.toml` file declaring the slice's prerequisites and interop shape. **Synthetic Python/Rust target trees that exercise the verb's AST-walk are not yet shipped**; the upstream PR (ARY-2031) will add them under `target_env_blocker/`, `target_mp_tokio/`, `target_safety_kernel/` paths referenced by the TOMLs once the verb implementation lands. Until then, the TOMLs themselves are the canonical fixture and the verb spec at `expected_output.txt` is scoped to "TOML-driven checks now, AST-walk-of-target later".

## Fixtures (TOML-only at v0)

| Slice TOML | Expected verdict | Why |
|------------|------------------|-----|
| `positive_env_blocker_slice.toml` | BLOCKED — env-prerequisite | declares a `torch.load(weights_only=True)` checkpoint prerequisite + a required HMAC env-var; the slice cannot start until both resolve |
| `positive_mp_to_tokio_slice.toml` | HIGH risk — cross-runtime | declares `source_concurrency = multiprocessing.Pool` / `target_concurrency = tokio::task` + 8 PyO3 call sites — globals not preserved fork→thread |
| `negative_safety_kernel_slice.toml` | LOW risk — clean | declares matched concurrency models (asyncio.Task → tokio::task), zero PyO3 sites, no prerequisites — negative control |

## Adversarial / negative-control discipline (Rule 9)

The verb MUST be advisory, not a hammer. A bug class on its own is meaningless — `multiprocessing` in the source is fine if the Rust target ALSO uses `multiprocessing.Pool` via PyO3 subprocess (same fork semantics). The risk pattern fires only when there's a **shape mismatch** between source and target. The negative-control fixture asserts the verb doesn't over-flag.

## Postmortem feedback loop (verb #4 interaction)

The `--use-postmortems` flag cross-references prior postmortems against the current slice. When `postmortem-record/positive/atlas-2026-05.toml` is loaded and `slice-risk --use-postmortems positive_env_blocker_slice.toml` runs, the verb MUST emit a cross-reference line:

```
Cross-referencing 1 prior postmortem:
  - 2026-05-phase-3d-atlas: env-prerequisite blocker
  - applies here: torch.load checkpoint compat (✗ same dependency detected)
```

The cross-reference relies only on the TOML metadata (`load_method`, `prevents_pattern`); no target-tree AST walk is required at v0. ARY-2031 will extend this to also walk the target tree when present.
