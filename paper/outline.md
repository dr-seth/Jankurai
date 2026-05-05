# Jankurai Outline

Paper edition: `2026.05-ed7`

## Sections

| Section | File | Status |
| --- | --- | --- |
| Frontmatter and abstract | `paper/tex/frontmatter.tex` | done |
| 1. From Vibe Coding to Verified Merge | `paper/tex/sections/01_new_bottleneck.tex` | done |
| 2. Running Example: Checkout PR | `paper/tex/sections/02_running_example.tex` | done |
| 3. Definitions and Threat Model | `paper/tex/sections/02_definitions_threat_model.tex` | done |
| 4. Jankurai Core Standard and Conformance | `paper/tex/sections/06_jankurai_standard.tex` | done |
| 5. Vibe-Artifact Taxonomy and Stable Rule IDs | `paper/tex/sections/05_fault_taxonomy.tex` | done |
| 6. Evaluation and Conformance Evidence | `paper/tex/sections/07_evaluation_and_conformance_suite.tex` | done |
| 7. Agent Repository Controls and Tool Adapters | `paper/tex/sections/10_agent_controls.tex` | done |
| 8. Continuous Proof: From Changed Paths to Merge Witness | `paper/tex/sections/11_continuous_proof.tex` | done |
| 9. Rendered UX and Browser-Step QA | `paper/tex/sections/12_pixel_qa.tex` | done |
| 10. Security, Supply Chain, and Permissions | `paper/tex/sections/13_security_permissions.tex` | done |
| 11. Waivers, Observability, and Repair Receipts | `paper/tex/sections/14_exceptions_repair.tex` | done |
| 12. Migration, Versioning, and Governance | `paper/tex/sections/15_migration_governance.tex` | done |
| 13. Languages as Proof-Cost Compression | `paper/tex/sections/03_language_compression.tex` | done |
| 14. Technical Promise Versus Standard Gravity | `paper/tex/sections/04_standard_gravity.tex` | done |
| 15. Non-Normative Reference Profile Score | `paper/tex/sections/07_stack_rubric.tex` | done |
| 16. Reference Profile Comparison | `paper/tex/sections/08_stack_ranking.tex` | done |
| 17. Reference Architecture Profile | `paper/tex/sections/09_winner_architecture.tex` | done |
| 18. Related Work | `paper/tex/sections/15_related_work.tex` | done |
| 19. Limitations and Research Agenda / Conclusion | `paper/tex/sections/16_limitations_conclusion.tex` | done |

## Appendices

| Appendix | File | Status |
| --- | --- | --- |
| Rule IDs and Conformance Evidence | `paper/tex/appendices/a_rule_ids.tex` | done |
| Versioned Artifact Manifest | `paper/tex/appendices/b_artifact_manifest.tex` | done |
| Waiver and Repair Templates | `paper/tex/appendices/c_exception_template.tex` | done |
| Reference-Profile File Tree Diagrams | `paper/tex/appendices/d_file_trees.tex` | done |
| Golden First-Hour Command Path | `paper/tex/appendices/e_command_map.tex` | done |

## Working Notes

- `paper/jankurai.tex` is a thin TeX wrapper and remains canonical.
- `paper/jankurai.md` is an agent companion, not a generator input.
- Paper artifacts use `jankurai.*`; do not create `main.*` paper files.
- Stack scoring is non-normative and must not reject otherwise conformant profiles.
