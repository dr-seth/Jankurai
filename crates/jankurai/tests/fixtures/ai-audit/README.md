# Verb #2 — `jankurai ai audit <dir>` fixtures

Verb: scan a directory tree for AI/LLM call sites, classify each into one of four tiers (INTEGRATED, CLASSIFY/WRAPPER, GENERATE/AGENT, FALSE_POSITIVE), output a per-site table.

Heuristics are tunable via `ai-audit.toml` (example included).

## Fixtures

| File | Expected tier | Why |
|------|---------------|-----|
| `source_fixtures/trading_nl_classify.py` | 2 — CLASSIFY (Anthropic flavor) | `temperature=0.1` + `max_tokens=200` + small enum enumerated in system prompt + JSON-shape instruction in prose → nano-replaceable. Does NOT use `response_format` (Anthropic's Messages API rejects that kwarg; it's OpenAI Chat Completions-specific). |
| `source_fixtures/trading_nl_generate.py` | 3 — GENERATE | `max_tokens=2000` + free-form user prompt → permanent LLM |
| `source_fixtures/channel_wrapper.py` | 2 — WRAPPER | system prompt enumerates 8 slash-commands → classification dressed as free-form |
| `source_fixtures/meta_rsi_agent.py` | 3 — AGENT | tool-use loop, generative |
| `source_fixtures/integrated_module.py` | 1 — INTEGRATED | imports arya_speaks bridge → already nano-routed; skip |
| `source_fixtures/oauth_falsepositive.py` | 4 — FALSE_POSITIVE | `"anthropic.com"` string literal in trusted-domains list, no `import anthropic` |
| `source_fixtures/llm_settings_falsepositive.py` | 4 — FALSE_POSITIVE | provider name as config-key string, no SDK import |

## Heuristics (excerpt from proposal §Contribution #2)

Heuristics are **provider-flavored** — Anthropic and OpenAI use different
APIs for structured output, so the verb must classify per SDK:

- **OpenAI Chat Completions**: `response_format={"type":"json_object"}` + small enum referenced in prompt → CLASSIFY (this fixture suite does not yet ship a positive example; would live in a hypothetical `trading_nl_classify_openai.py`)
- **Anthropic Messages**: `temperature` ≤ 0.3 + `max_tokens` ≤ 500 + closed-set enum in system prompt + explicit JSON-shape instruction → CLASSIFY (see `trading_nl_classify.py`)
- **Either flavor**: `max_tokens` > 1000 + free-form user content → GENERATE
- Module imports BOTH `anthropic` AND `openai` with provider switch → WRAPPER
- `system_prompt` enumerates a small closed set → likely WRAPPER dressed as GENERATE
- Module already imports project's nano-bridge → INTEGRATED, skip
- SDK package name appears only in a string literal, no import → FALSE_POSITIVE

**Important — Anthropic API surface**: the Anthropic Messages API does NOT
accept `response_format`. Fixtures that mix `from anthropic import ...` with
`response_format={"type":"json_object"}` are demonstrating an impossible
call shape and the verb should flag them (a future adversarial fixture).

## Adversarial assertion (Rule 9)

The verb MUST NOT classify by regex-matching SDK package names alone. Adversarial fixture `oauth_falsepositive.py` contains the string `"anthropic.com"` but no `import anthropic`. A regex-only verb misclassifies as Tier 1/2/3. The verb must:

1. Parse imports (AST)
2. Resolve the symbol at each potential call site
3. Confirm the call is *invoked*, not just present as a string literal
