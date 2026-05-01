# Agent Readiness Index Worksheet

Score each dimension from `0` to `5`. Use `0` for absent, `3` for usable but inconsistent, and `5` for reliable, documented, and exercised in real work.

| Dimension | Score | Evidence |
| --- | ---: | --- |
| Locality and navigability |  |  |
| Context economy |  |  |
| Executability |  |  |
| Contract clarity |  |  |
| Testability |  |  |
| Observability and remediation |  |  |
| Security posture |  |  |
| Build-loop speed |  |  |
| Maintainability under change |  |  |

## Caps

- Overall score cannot exceed `3` without one-command setup.
- Overall score cannot exceed `3` without a deterministic fast lane.
- Overall score cannot exceed `3` for high-risk systems without an explicit security lane.
- Overall score cannot exceed `4` when generated contracts or public API drift are untested.

## Formula

Default score is the unweighted average of the nine dimensions, then capped by the rules above. Teams may add weights, but the weights must be written here before scoring.

