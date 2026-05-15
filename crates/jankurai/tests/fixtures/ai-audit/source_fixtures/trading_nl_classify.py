"""Synthetic ai-audit fixture: Tier 2 CLASSIFY case (Anthropic flavor).

Expected: ai-audit classifies the call site at the line marked CLASSIFY_SITE
as Tier 2 (nano-replaceable). Heuristic hits applicable to Anthropic's
Messages API: temperature <= 0.3, max_tokens <= 500, small enum vocabulary
enumerated in system prompt with explicit JSON-shape instruction.

Note: deliberately does NOT pass `response_format={"type": "json_object"}`.
That kwarg is OpenAI Chat Completions-specific; Anthropic's Messages API
rejects it. Anthropic users get structured output via system-prompt
instructions (this fixture) or tool schemas (see meta_rsi_agent.py for the
tool-use shape). The OpenAI-flavored CLASSIFY heuristic with response_format
is covered separately by trading_nl_classify_openai.py if/when added.
"""

from anthropic import Anthropic

client = Anthropic()

INTENT_VOCAB = [
    "buy_market",
    "sell_market",
    "buy_limit",
    "sell_limit",
    "cancel_order",
    "show_positions",
    "show_balance",
    "transfer_funds",
    "subscribe_quote",
    "unsubscribe_quote",
]


def classify_trading_intent(user_text: str) -> str:
    """Classify a user utterance into one of INTENT_VOCAB."""
    system_prompt = (
        "You are a trading-intent classifier. Output exactly one of the following "
        "intent codes: " + ", ".join(INTENT_VOCAB) + ". Respond with a JSON object "
        'of shape {"intent": "<code>"} and nothing else.'
    )
    # CLASSIFY_SITE — this is the line ai-audit must classify as Tier 2.
    response = client.messages.create(
        model="claude-haiku-4-5",
        max_tokens=200,
        temperature=0.1,
        system=system_prompt,
        messages=[{"role": "user", "content": user_text}],
    )
    return response.content[0].text
