# humanlint Boundaries

The humanlint standard rejects ambiguous ownership. Each layer must own one
kind of truth and must not leak into adjacent layers.

| Layer | Owns | Must not own |
| --- | --- | --- |
| TypeScript web | UI, forms, route state, local validation, generated clients | secrets, durable truth, direct DB writes, core authz |
| Rust API | transport edge, request normalization, response mapping | domain rules hidden in handlers, scattered SQL |
| Rust domain | IDs, invariants, state machines, pure decisions | I/O, env, time, random, DB, framework types |
| Rust application | commands, authz, idempotency, transactions | UI concerns, provider-specific adapter details |
| Rust adapters | PostgreSQL, queues, external APIs, filesystem, env | domain rules |
| PostgreSQL | constraints, migrations, indexes, transactional truth | app orchestration |
| Python AI service | models, embeddings, evals, data transforms | product truth, authz, direct production DB writes |
| Ops/security | CI, OTel, SBOM, SCA, secret scanning, provenance | hidden product logic |

Boundary exceptions belong in `docs/exceptions/` with owner, reason,
expiration, proof lane, and repair guidance.
