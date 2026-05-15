"""Synthetic ai-audit fixture: Tier 1 INTEGRATED case.

Expected: ai-audit classifies as Tier 1 (already nano-routed; skip in summary).
Heuristic: module imports the project's nano-bridge (arya_speaks) and routes
through it before any direct SDK call.
"""

from arya_speaks.language_core import LLMConfig
from arya_speaks.language_core.llm_adapter import ARYASpeaksProvider


def get_intent_provider(vocab: list[str]) -> ARYASpeaksProvider:
    """Return a provider configured with the caller's vocabulary."""
    config = LLMConfig.for_arya_speaks(
        caller_vocabulary=vocab,
        caller_vocabulary_mode="semantic",
    )
    return ARYASpeaksProvider(config)


async def classify(user_text: str, vocab: list[str]) -> str:
    """Classify via arya-speaks; falls back to LLM only on abstain (handled
    inside ARYASpeaksProvider)."""
    provider = get_intent_provider(vocab)
    # INTEGRATED_SITE — ai-audit sees the nano bridge and skips this site.
    resp = await provider.complete_async(
        messages=[{"role": "user", "content": user_text}],
        json_mode=True,
    )
    return resp.chosen_atom_id
