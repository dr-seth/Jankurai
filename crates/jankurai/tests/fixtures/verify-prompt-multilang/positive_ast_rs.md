# AARA Engine (Rust) — scoping (POSITIVE fixture, deeper-AST)

> v2 names every Rust symbol correctly and points at a real parsed
> item, so the verify-prompt verb (with the deeper-AST cross-check)
> should accept it (decision=pass).

## Hot-path entry points

- `sense` at `synthetic_rust_engine_stub.rs:9` — real `impl` method, present in the syn AST.
- `real_top_level` at `synthetic_rust_engine_stub.rs:27` — genuine top-level function, present in the syn AST.
