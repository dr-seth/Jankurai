# AARA Engine (Rust) — scoping (ADVERSARIAL fixture, deeper-AST)

> SCRUBBED extract of an aspirational doc whose Rust claims are
> regex-spoofable. The slice-2 regex resolver alone would VERIFY the
> first claim below (a `fn` header that physically appears at the
> claimed line). The deeper-AST upgrade (ARY-2029) parses the whole
> file with `syn` and must REJECT it, because the line lives inside a
> string literal — it is not a parsed item.

## Hot-path entry points (CLAIMED — actually spoofed / wrong)

- `ghost_decide` at `synthetic_rust_engine_stub.rs:20` — claims a method definition. The regex sees `fn ghost_decide` at L20, but syn parses L19-21 as a string literal; `ghost_decide` is not a real item. Must be rejected as a regex-only symbol.
- `snse` at `synthetic_rust_engine_stub.rs:9` — typo claim. L9 actually defines `sense`; the AST cross-check must report a symbol mismatch with the nearest syn item as the suggestion.
