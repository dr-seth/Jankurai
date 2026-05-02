# Jankurai Moonshot

Jankurai exists to make agent-native engineering boring in the right way. The repo should tell an agent what it owns, how to prove a change, where generated truth lives, and when an exception must expire.

The operating loop is:

```text
intent -> bounded agents -> proof lanes -> evidence -> expired exceptions -> reusable primitives
```

That loop is the product, the standard, and the paper thesis at the same time. It changes the center of gravity from "can a human keep all of this in their head?" to "can the repo prove the change with the smallest credible lane?"

## What The Moonshot Demands

- intent is explicit before code starts moving
- agents get bounded authority, not blanket permission
- every changed path maps to ownership and proof
- generated outputs have declared sources and regeneration commands
- evidence is durable, machine-readable, and easy to audit
- exceptions are versioned, owned, and time-bounded
- repeated fixes become reusable primitives instead of copy-paste folklore

## What It Rejects

- ambiguous ownership
- hidden fallback behavior
- handwritten mirrors of contracts and generated types
- UI, Python, or adapter layers that quietly own durable product truth
- generated files edited by hand
- proof that passes without covering the changed behavior
- exceptions that never expire

## Why It Matters

Agent-generated code is cheap. Wrong code is also cheap. The new bottleneck is trustworthy merge: the repository has to make the correct path obvious, the proof lane small, the evidence durable, and the repair local.

Jankurai is the control plane for that workflow. It does not replace taste, product judgment, or human accountability. It gives those decisions a machine-readable boundary so agents can act quickly without turning the repo into a guessing game.

## Practical Consequences

- Humans supply intent, constraints, and risk tolerance.
- Jankurai routes the change to the smallest proof lane that covers the risk.
- The audit emits JSON and Markdown so humans and agents see the same truth.
- A temporary exception can exist, but it must carry an owner and an expiry path.
- Repeated patterns should collapse into reusable cells, templates, or generated primitives.

## Success Criteria

If the moonshot is working, a fresh agent can open the repo, find the owner, choose the proof lane, run the checks, and leave evidence without learning the whole codebase first. Wrong code becomes easier to reject than to rationalize.
