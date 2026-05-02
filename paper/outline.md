# jankurai Outline

Status is tracked by TeX source file so the canonical paper can be reviewed without reopening the planning thread.

Paper edition: `2026.05-ed3`

## Sections

| Section | File | Status |
| --- | --- | --- |
| Frontmatter and abstract | `paper/tex/frontmatter.tex` | done |
| 1. New Bottleneck: Trustworthy Merge | `paper/tex/sections/01_new_bottleneck.tex` | done |
| 2. Definitions and Threat Model | `paper/tex/sections/02_definitions_threat_model.tex` | done |
| 3. Languages as Bottleneck Compression | `paper/tex/sections/03_language_compression.tex` | done |
| 4. Technical Promise Versus Standard Gravity | `paper/tex/sections/04_standard_gravity.tex` | done |
| 5. Vibe-Coding Fault Taxonomy | `paper/tex/sections/05_fault_taxonomy.tex` | done |
| 6. jankurai Standard and Conformance | `paper/tex/sections/06_jankurai_standard.tex` | done |
| 7. A 100-Point Stack Rubric | `paper/tex/sections/07_stack_rubric.tex` | done |
| 8. Stack Ranking, Sensitivity, and Exceptions | `paper/tex/sections/08_stack_ranking.tex` | done |
| 9. Default Winner Architecture | `paper/tex/sections/09_winner_architecture.tex` | done |
| 10. Agent Repository Controls and Tool Adapters | `paper/tex/sections/10_agent_controls.tex` | done |
| 11. Continuous Proof, CI, and Test Explosion | `paper/tex/sections/11_continuous_proof.tex` | done |
| 12. Rendered UX and Browser-Step QA | `paper/tex/sections/12_pixel_qa.tex` | done |
| 13. Security, Supply Chain, and Permissions | `paper/tex/sections/13_security_permissions.tex` | done |
| 14. Exceptions, Observability, and Repair Receipts | `paper/tex/sections/14_exceptions_repair.tex` | done |
| 15. Migration, Versioning, and Governance | `paper/tex/sections/15_migration_governance.tex` | done |
| 16. Limitations and Research Agenda / Conclusion | `paper/tex/sections/16_limitations_conclusion.tex` | done |

## Appendices

| Appendix | File | Status |
| --- | --- | --- |
| Rule IDs and Conformance Evidence | `paper/tex/appendices/a_rule_ids.tex` | done |
| Versioned Artifact Manifest | `paper/tex/appendices/b_artifact_manifest.tex` | done |
| Exception and Repair Templates | `paper/tex/appendices/c_exception_template.tex` | done |

## Working Notes

- `paper/jankurai.tex` is a thin TeX wrapper and remains canonical.
- `paper/jankurai.md` is an agent companion, not a generator input.
- Paper artifacts use `jankurai.*`; do not create `main.*` paper files.
- `paper/references.bib` is the citation spine for the integrated manuscript.
- `paper/citation-index.md` is the claim-to-source ledger.
