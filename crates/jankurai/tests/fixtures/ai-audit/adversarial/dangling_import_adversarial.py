"""Rule-9 / Rule-8 adversarial fixture: dangling import, NO call site.

Imports `anthropic` at module level but never invokes
`client.messages.create(...)`. A regex-name-match auditor would flag this
as an LLM call site. The verb MUST re-derive from call-shape inspection
and emit `no_call_site` (tier excluded from the tier-1..4 summary).
"""

import anthropic  # module-level import, but never called

_VERSION = anthropic.__version__


def describe_sdk() -> str:
    # No `.messages.create(` anywhere — only a version string read.
    return f"anthropic sdk {_VERSION}"
