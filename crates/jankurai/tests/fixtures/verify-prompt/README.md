# Verb #1 — `jankurai migrate verify-prompt <doc>` fixtures

Verb: parse a migration doc, extract every `<file>:<line>` / `<module>::<function>` / `class <Name>(<Base>)` claim, resolve to source, and verify file/line/symbol/shape. Exit non-zero with each unverified claim listed.

## Fixtures

| File | Role | Expected verb behaviour |
|------|------|------------------------|
| `negative_v1.md` | Adversarial: scoping doc with 8 call-site claims — 7 falsifiable, 1 true (the `_act` reference at L1171, which resolves correctly and demonstrates the verb's discrimination) | Verb MUST exit 1, surfacing the 7 falsifiable claims + 1 partial/info entry for the true claim |
| `positive_v2.md` | Positive: the v2 doc, falsifiable claims removed | Verb MUST exit 0 |
| `synthetic_engine_stub.py` | Source-resolution target: synthetic stub of `packages/aara/engine.py` with matching line numbers + callable signatures. Function bodies are `pass`-stubs **EXCEPT** `_extract_answer_from_result` at L1693, whose ~11-statement dict-extractor body is preserved verbatim so the v2 doc's shape claim ("24-line dict extractor, no SDK imports") can be re-derived from AST | Used by the verb to resolve `engine.py:NNN` claims |
| `synthetic_aara_api_stub.py` | Source-resolution target: stub of `packages/aara/api.py` for the `invoke()` return-type claim | Same as above |
| `expected_output.txt` | Spec for the test asserter | Exact pass/fail diagnostic lines |

## How the verb re-derives evidence (Rule 9)

For each claim in the doc, the verb computes a re-derivation:

| Claim shape | Re-derivation | Pass condition |
|-------------|--------------|----------------|
| `<file>:<line>` is `<description>` | Read the file; locate the callable enclosing that line via AST; compute heuristic shape (line count, presence of SDK imports in body, presence of structured return) | Heuristic shape matches `<description>` keywords |
| `<module>.<symbol>` exists | AST walk module, look up symbol | Symbol found in module's symbol table |
| `class <Name>(<Base>)` | AST walk, find ClassDef where name=Name | Base resolves to Base |

Regex-matching the file content for the description text is **forbidden** — that fails Rule 9. The re-derivation must run an actual AST resolution.

## Scrubbing notes

- `negative_v1.md` and `positive_v2.md` are derived from the public proposal doc and its referenced files. References to specific file paths (`packages/aara/engine.py`) are kept because they appear in the public proposal already.
- `synthetic_engine_stub.py` has the same function names + line numbers as the real module, but bodies are `pass`-stubs for `_sense`, `_decide`, `_act`, `_verify`. The single exception is `_extract_answer_from_result` at L1693, whose ~11-statement dict-extractor body is preserved verbatim — required by the validator's shape check (`stmt_count` in 3..12 + no SDK imports in body) and load-bearing for the v2 doc's shape-mismatch claim against v1. A `pass`-stub there would fail the validator.
