"""Synthetic ai-audit fixture: Tier 3 GENERATE case.

Expected: ai-audit classifies the call site as Tier 3 (permanent LLM).
Heuristic hits: max_tokens=2000, free-form user prompt, no enum.
"""

from anthropic import Anthropic

client = Anthropic()


def explain_strategy_to_user(user_question: str, strategy_context: str) -> str:
    """Generate a free-form NL explanation of a trading strategy."""
    system_prompt = (
        "You are a thoughtful explainer of trading strategies. Answer the user's "
        "question using the provided context. Be precise and educational."
    )
    # GENERATE_SITE — ai-audit must classify as Tier 3.
    response = client.messages.create(
        model="claude-opus-4-7",
        max_tokens=2000,
        temperature=0.7,
        system=system_prompt,
        messages=[
            {"role": "user", "content": user_question},
            {"role": "assistant", "content": strategy_context},
        ],
    )
    return response.content[0].text
