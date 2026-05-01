# Stack-Ranking Figure

Source data for `stack-ranking.eps`.

Use in LaTeX:

```tex
\includegraphics[width=\linewidth]{figures/stack-ranking.eps}
```

## Rubric Scores

| Rank | Stack | Score |
|---:|---|---:|
| 1 | Rust core + TS/React/Vite + PostgreSQL | 94 |
| 2 | Go services + TS/React/Vite + PostgreSQL | 90 |
| 3 | C#/.NET + TS/React/Vite + PostgreSQL/SQL Server | 89 |
| 4 | TS product plane + Rust/Go compute cells + PostgreSQL | 88 |
| 5 | Kotlin/Java JVM + TS/React/Vite + PostgreSQL/Kafka | 87 |

## Design Notes

- EPS is hand-authored PostScript with no external image dependency.
- Bounding box is `0 0 840 470`.
- The x-axis starts at `69.6`, which is 80% of the lowest score (`87`).
- The PDF is generated from the EPS with `epstopdf`.
