"""Synthetic ai-audit fixture: Tier 4 FALSE_POSITIVE case (settings shape).

Expected: ai-audit classifies as Tier 4. Provider names appear as config keys
in a settings dict; no SDK import, no call.
"""

# Note: no `import anthropic`, no `import openai`. This is settings only.

LLM_PROVIDER_DEFAULTS = {
    "anthropic": {
        "model_default": "claude-opus-4-7",
        "max_tokens_default": 1024,
        "timeout_seconds": 30,
    },
    "openai": {
        "model_default": "gpt-4",
        "max_tokens_default": 1024,
        "timeout_seconds": 30,
    },
    "groq": {
        "model_default": "llama-3-70b",
        "max_tokens_default": 1024,
        "timeout_seconds": 30,
    },
}


def get_default_max_tokens(provider: str) -> int:
    return LLM_PROVIDER_DEFAULTS.get(provider, {}).get("max_tokens_default", 256)
