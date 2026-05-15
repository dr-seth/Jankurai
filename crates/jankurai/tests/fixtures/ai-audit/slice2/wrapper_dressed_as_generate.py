"""ai-audit slice-2 fixture: WRAPPER dressed as GENERATE (disambiguation).

A large max_tokens budget (2000, >= generate_max_tokens_min) would, on a
naive reading, look like a permanent generative call (Tier 3 GENERATE).
But the system prompt enumerates a small CLOSED set of options — the call
is really a classifier wearing a free-form costume. The verb MUST resolve
this to WRAPPER (Tier 2, nano-replaceable), not GENERATE: the
enum-options heuristic is evaluated BEFORE the max_tokens GENERATE
heuristic.

Expected: WRAPPER, tier_2.
"""

from anthropic import Anthropic

client = Anthropic()

ROUTES = [
    "/triage",
    "/escalate",
    "/resolve",
    "/snooze",
    "/reassign",
    "/merge",
]


def route_ticket(ticket_text: str) -> str:
    """Pick a route for an incoming support ticket."""
    system_prompt = (
        "You are a support-ticket router. Choose exactly one of the "
        "available routes and explain your reasoning at length: "
        + ", ".join(ROUTES)
        + ". Be thorough and discursive in the justification."
    )
    # WRAPPER_SITE — big max_tokens, but a closed enumerated set => Tier 2.
    response = client.messages.create(
        model="claude-opus-4-7",
        max_tokens=2000,
        temperature=0.6,
        system=system_prompt,
        messages=[{"role": "user", "content": ticket_text}],
    )
    return response.content[0].text
