# AARA Engine × arya-speaks — replacement scoping (v2, POSITIVE fixture)

> SCRUBBED, MINIMAL extract of the real v2 scoping doc at
> `docs/migration/aara-engine-arya-speaks-scoping-v2.md`. v2 removed every
> falsifiable claim that v1 made. The verify-prompt verb should accept
> this doc (exit 0).

## Hot-path entry points

The four phase methods of `AARAEngine` (`_sense`, `_decide`, `_act`, `_verify`) and the post-ACT answer extractor live in `synthetic_engine_stub.py`:

- `_sense` (`synthetic_engine_stub.py:386` — phase 1; method def directly after line 382's comment block)
- `_decide` (`synthetic_engine_stub.py:674` — phase 2; capability-match method)
- `_act` (`synthetic_engine_stub.py:1171` — phase 3; dispatcher entry point)
- `_verify` (`synthetic_engine_stub.py:1476` — phase 4; verification phase)
- `_extract_answer_from_result` (`synthetic_engine_stub.py:1693` — NOT a phase method; post-ACT answer extractor; ~24-line dict-key extractor; NOT an LLM call)

What matters here: every file/line above resolves to a non-trivial line in
`synthetic_engine_stub.py`, and `_extract_answer_from_result` is correctly
described as a dict extractor (matching its AST shape).

## Public APIs

- The atom extractor's real entry point is `EAN.predict(text, threshold, top_k)`
  in `arya_speaks.language_core` — not `EAN.extract_atoms`. (This claim is
  about an external module; verify-prompt SHOULD treat it as unresolvable from
  fixture-local sources and surface it as an `external-module` warning, not an
  error — that is the v2 doc's whole point: don't make falsifiable claims about
  modules you can't ship in the fixture.)
- `AARAEngine.invoke()` (`synthetic_aara_api_stub.py:170`) returns the action
  dict verbatim (not NL prose).

## What needs to change

The work proposed in v1 is still desirable; v2 names it correctly.
