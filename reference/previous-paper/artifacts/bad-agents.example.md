# AGENTS.md

Read every file before changing anything. Always run the entire test suite. Prefer clever abstractions. Use your judgment about generated files. If tests are noisy, summarize them however seems useful. Security is important, so be careful. This repo has many crates and most things are connected. Try to keep changes small but fix adjacent issues if you notice them. The database and API schemas should stay in sync. Ask before doing dangerous things unless you think it is safe.

## Why This Is Bad

- It does not route ownership.
- It gives no canonical proof lanes.
- It tells the agent to over-read.
- It hides generated-file authority.
- It gives no raw-output or exit-code policy.
- It makes security vague instead of executable.
