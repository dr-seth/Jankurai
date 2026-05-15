"""ai-audit slice-2 fixture: Tier 2 CLASSIFY — OpenAI flavor.

The slice-1 suite only shipped the Anthropic-flavor CLASSIFY
(`trading_nl_classify.py`, structured output via system-prompt JSON
instruction). OpenAI's Chat Completions API instead uses
`response_format={"type": "json_object"}`. The verb's CLASSIFY heuristic
accepts EITHER a prose JSON-shape instruction (Anthropic) OR
`response_format` (OpenAI) — this fixture exercises the OpenAI branch.

Expected: CLASSIFY, tier_2 (temperature<=0.3, max_tokens<=500, closed-set
enum referenced in the system prompt, response_format=json_object).
"""

from openai import OpenAI

client = OpenAI()

INTENT_VOCAB = [
    "buy_market",
    "sell_market",
    "buy_limit",
    "sell_limit",
    "cancel_order",
    "show_positions",
    "show_balance",
]


def classify_trading_intent(user_text: str) -> str:
    """Classify a user utterance into one of INTENT_VOCAB (OpenAI flavor)."""
    system_prompt = (
        "You are a trading-intent classifier. Output exactly one of the "
        "following intent codes: " + ", ".join(INTENT_VOCAB) + "."
    )
    # CLASSIFY_SITE — OpenAI structured-output shape: response_format json.
    response = client.chat.completions.create(
        model="gpt-4o-mini",
        max_tokens=200,
        temperature=0.1,
        response_format={"type": "json_object"},
        messages=[
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_text},
        ],
    )
    return response.choices[0].message.content
