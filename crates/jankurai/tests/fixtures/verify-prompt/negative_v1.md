# AARA Engine × arya-speaks — replacement scoping (v1, ADVERSARIAL fixture)

> Source-of-record: this is a SCRUBBED, MINIMAL extract of the real v1
> scoping doc that shipped in QantmOrchstrtr-RSI at
> `docs/migration/aara-engine-arya-speaks-scoping.md` (commit `783d9dc54`'s
> parent). It contains the 7 known-false call-site claims that an
> aspirational-spec doc made about `packages/aara/engine.py`. The verify-prompt
> verb must surface each one.

## Hot-path entry points (CLAIMED — actually all wrong)

The four phase methods of `AARAEngine` are the LLM-fronted boundary in the hot path:

- `AARAEngine._sense_formalize` at `synthetic_engine_stub.py:382` — LLM-formalizes the request into a typed problem statement.
- `AARAEngine._decide_route_via_llm` at `synthetic_engine_stub.py:674` — LLM-driven capability matcher.
- `AARAEngine._act` at `synthetic_engine_stub.py:1171` — Dispatches to a `GovernedUnit` and may call out via tool-use to LLM.
- `AARAEngine._verify_with_llm_judge` at `synthetic_engine_stub.py:1476` — LLM-as-judge verifier.
- `AARAEngine._extract_answer_from_result` at `synthetic_engine_stub.py:1693` — Final NL summarization call; produces the prose response.

## Public APIs (CLAIMED — actually wrong)

- The atom extractor is exposed as `EAN.extract_atoms(text, threshold)` in `arya_speaks.language_core`.
- `AARAEngine.invoke()` (`synthetic_aara_api_stub.py:170`) returns the NL prose answer as a string.
- `SpeaksLanguageCore` lives at `synthetic_speaks_bridge_stub.py` (path claim — actual path differs).

## What needs to change

Each LLM call site above should route through `SpeaksLanguageCore` first, falling back to LLM only on abstain.
