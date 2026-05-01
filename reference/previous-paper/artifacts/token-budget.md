# Token Budget Template

Use this file to make token cost reviewable. Token savings are acceptable only when correctness, security, auditability, and human review quality are preserved.

## Task

- Issue or change ID:
- Owner crate/module:
- Risk class: low / medium / high / security-sensitive
- Required proof lane: fast / medium / deep / security / release

## Budget

| Bucket | Budget | Actual | Notes |
| --- | ---: | ---: | --- |
| Instructions | | | Root and path-local guidance only |
| Navigation/search | | | `rg`, repo map, Cargo metadata |
| File reads | | | Prefer owner files and signature-first reads |
| Reasoning | | | Include repair-plan tokens if tracked |
| Patch | | | Diff and edit tokens |
| Proof output | | | Compressed summary plus raw log path |
| Wrong turns | | | Wrong owner, wrong theory, broad patch |
| Recovery | | | Follow-up after failed proof |
| Human handoff | | | Review packet, summary, links |

## Acceptance

- Exit codes are preserved for every compressed command.
- Full raw output path is included for every compressed summary.
- Secret material is redacted before model exposure.
- Security failures are never summarized as green.
- If hidden/security validation fails, token savings do not count as success.
