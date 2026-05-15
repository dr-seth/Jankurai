"""Synthetic ai-audit fixture: Tier 2 WRAPPER case.

Expected: ai-audit classifies as Tier 2 (replaceable). Despite max_tokens=800
and absence of response_format=json_object, the heuristic
`system_prompt enumerates a small closed set of options` should fire.
"""

from anthropic import Anthropic
from openai import OpenAI  # noqa: F401 — used to demonstrate provider-switch shape

anthropic_client = Anthropic()


SLASH_COMMANDS = [
    "/help",
    "/quote",
    "/buy",
    "/sell",
    "/positions",
    "/balance",
    "/news",
    "/settings",
]


def route_channel_message(text: str) -> str:
    """Map a user channel message to one of SLASH_COMMANDS."""
    system_prompt = (
        "You route channel messages. Available commands are: "
        + ", ".join(SLASH_COMMANDS)
        + ". Reply with exactly one of these commands."
    )
    # WRAPPER_SITE — ai-audit must classify as Tier 2 (closed-set system prompt
    # is classification dressed as free-form).
    response = anthropic_client.messages.create(
        model="claude-haiku-4-5",
        max_tokens=800,
        temperature=0.0,
        system=system_prompt,
        messages=[{"role": "user", "content": text}],
    )
    return response.content[0].text
