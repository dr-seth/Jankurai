# AARA Engine (TypeScript port) — scoping (v2, POSITIVE fixture)

> v2 names every TS symbol correctly so the verify-prompt verb (with the
> slice-2 multi-lang resolver) should accept it (decision=pass).

## Hot-path entry points

- `AARAEngine.sense` at `synthetic_aara_engine_stub.ts:20` — class method.
- `AARAEngine.decide` at `synthetic_aara_engine_stub.ts:30` — async class method.
- `AARAEngine.act` at `synthetic_aara_engine_stub.ts:36` — public class method.
- `AARAEngine.verifyResult` at `synthetic_aara_engine_stub.ts:42` — private class method.

## Public APIs

- `extractAtoms` at `synthetic_aara_engine_stub.ts:51` — top-level function.
- `buildEngine` at `synthetic_aara_engine_stub.ts:57` — arrow-binding constant.
