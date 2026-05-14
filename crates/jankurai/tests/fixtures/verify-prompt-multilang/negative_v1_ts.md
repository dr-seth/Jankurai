# AARA Engine (TypeScript port) — scoping (v1, ADVERSARIAL fixture)

> SCRUBBED extract of an aspirational v1 doc that makes wrong-symbol
> claims about `packages/aara/AARAEngine.ts`. The verify-prompt verb,
> after the slice-2 multi-lang upgrade, must surface each one — slice-1
> only knew about Python `def` / Rust `fn` headers and would silently
> verify TS class methods.

## Hot-path entry points (CLAIMED — actually wrong)

- `AARAEngine.senseLlmFormalize` at `synthetic_aara_engine_stub.ts:20` — claims an LLM-formalize method, but the actual method at L20 is `sense`.
- `AARAEngine.decideLlmRoute` at `synthetic_aara_engine_stub.ts:30` — claims a different method name; the actual async method at L30 is `decide`.
- `AARAEngine.act` at `synthetic_aara_engine_stub.ts:36` — TRUE claim (the only one). `act` really is at L36.
- `AARAEngine.verify` at `synthetic_aara_engine_stub.ts:42` — claim says `verify`; the actual private method at L42 is named `verifyResult`.

## Public APIs (CLAIMED — actually wrong)

- `extractAtomsWithThreshold` at `synthetic_aara_engine_stub.ts:51` — top-level fn claim; actual top-level fn at L51 is `extractAtoms` (no `WithThreshold` suffix).
- `buildEngineFromConfig` at `synthetic_aara_engine_stub.ts:57` — arrow-binding claim; actual binding at L57 is `buildEngine`.
