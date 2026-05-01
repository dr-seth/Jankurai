# RTK Integration Notes

RTK is treated as project-reported practitioner/tooling evidence, not peer-reviewed evidence.

## Use Case

Use RTK or an equivalent wrapper when command output is large, repetitive, and mostly irrelevant to the next repair step.

## Required Properties

- Preserve the original exit code.
- Write the full raw output to a stable file.
- Expose the raw file path in the compressed summary.
- Redact secrets before model exposure.
- Never summarize security failures as green.
- Do not remove the first failing compiler diagnostic, panic, assertion, advisory, or denied dependency finding.

## Rust-Specific Filters

- Group `cargo check --message-format=json` diagnostics by package, target, error code, and primary span.
- Collapse repeated borrow-checker follow-on notes after preserving the primary error and one representative help note.
- Show only failing nextest tests by default, with a raw-output link for all skipped/passed tests.
- Summarize `cargo tree` by duplicate versions, feature activations, and security-relevant dependencies.
- Summarize tracing logs by span path, error code, and first failure cause.

## Failure Policy

If a human or agent cannot reproduce the failure from the summary plus raw-output path, the compression is invalid and must be disabled for that command.
