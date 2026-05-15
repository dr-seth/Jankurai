"""Synthetic ai-audit fixture: Tier 3 AGENT case.

Expected: ai-audit classifies as Tier 3 (permanent LLM). Tool-use loop with
generative free-form reasoning between steps — not a closed-set classifier.
"""

from anthropic import Anthropic

client = Anthropic()

TOOLS = [
    {"name": "search", "description": "Search the knowledge graph."},
    {"name": "compute", "description": "Run a deterministic computation."},
    {"name": "propose_hypothesis", "description": "Emit a new hypothesis to test."},
]


def meta_rsi_step(scratchpad: str, observations: list[dict]) -> dict:
    """One step of the RSI reasoning loop."""
    system_prompt = (
        "You are a meta-RSI reasoning agent. Use the available tools to test "
        "hypotheses and propose new ones. Continue until you have a verified "
        "improvement to the running model."
    )
    # AGENT_SITE — ai-audit must classify as Tier 3.
    response = client.messages.create(
        model="claude-opus-4-7",
        max_tokens=2000,
        temperature=0.5,
        system=system_prompt,
        tools=TOOLS,
        messages=[
            {"role": "user", "content": scratchpad},
            *[{"role": "user", "content": str(o)} for o in observations],
        ],
    )
    return {"content": response.content, "stop_reason": response.stop_reason}
