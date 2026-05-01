# humanlint White Paper

Paper title: **Humans Were the Bug: From Vibe Coding to Agent-Native Engineering**

Canonical typeset source: `paper/humanlint.tex`

Compiled artifact: `paper/humanlint.pdf`

## Abstract

AI coding changes the primary failure mode of software organizations. The bottleneck is no longer whether a human knows a language well enough to type plausible code. The bottleneck is whether the repository can reject wrong generated code quickly, localize the defect, prove the repair, preserve security posture, and teach the next agent what happened.

`humanlint` proposes an agent-native standard centered on Rust core, TypeScript/React/Vite product surface, PostgreSQL durable truth, generated contracts, and bounded Python for AI/data service work. The stack is chosen for compile-time rejection, contract enforcement, database truth, deterministic testing, security audit, observability, and repair routing.

## Structure

1. Introduction
2. Languages as Compression Technology
3. The Harsh AI Reality
4. Method: A 100-Point Stack Rubric
5. Top-Five Stack Ranking
6. Winner-Only Architecture
7. Agent-First Repository Design
8. Testing and Automated QA
9. Agent-Friendly Exceptions
10. The humanlint Audit
11. Migration, Versioning, and Future Research
12. Conclusion

## Core Claims

- Agent-generated code should be treated as a bounded hypothesis until proof lanes accept it.
- Language choice now favors verification, security, contracts, and repair locality over author comfort.
- The default stack should be Rust core, TypeScript/React/Vite surface, PostgreSQL truth, generated contracts, and bounded Python.
- The repository must become a verification interface through owner maps, test maps, generated zones, proof lanes, agent-friendly exceptions, and CI audit output.
- Product/runtime code should not normalize future-hostile terms such as `legacy`, `deprecated`, `temporary`, `fallback`, `stub`, and `TODO` unless they are product copy or documented exceptions.

## Build

```bash
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/humanlint.tex
```

The bibliography is embedded through `paper/references.bib`. The citation ledger lives in `paper/citation-index.md`.
