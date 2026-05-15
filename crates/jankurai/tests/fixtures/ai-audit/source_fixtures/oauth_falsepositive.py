"""Synthetic ai-audit fixture: Tier 4 FALSE_POSITIVE case.

Expected: ai-audit classifies as Tier 4 (false positive).

Trap: the string "anthropic.com" appears in a config list. A regex-only
SDK-name search will flag this file; the AST resolver MUST see that there
is no `import anthropic` and no `Anthropic()` call site, only a string.
"""

# Note: no `import anthropic` here. No SDK usage.

TRUSTED_OAUTH_DOMAINS = [
    "google.com",
    "github.com",
    "linkedin.com",
    "anthropic.com",   # <-- trap: ai-audit must NOT flag this as an LLM site
    "openai.com",      # <-- trap: ditto
    "microsoft.com",
]


def is_trusted_domain(domain: str) -> bool:
    return domain in TRUSTED_OAUTH_DOMAINS
