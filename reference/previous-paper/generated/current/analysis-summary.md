# Analysis Summary for main-bare-vs-proof-full-20260401T001136Z

- Model: gpt-5.3-codex-spark
- Effort: high
- Conditions: bare, proof-full
- Issues: ISSUE-01-entitlement-grace, ISSUE-03-webhook-incident, ISSUE-04-entitlement-recovery-replay
- Runset complete: True
- Expected runs: 18, Actual runs: 18

## Solved-run medians
- bare: median tokens=275134, ETTS=497746
- proof-full: median tokens=264442, ETTS=413441

## Overall condition summary
- bare: median tokens=275134, ETTS=497746, wrong-turn tax=211642, owner convergence=1.00
- proof-full: median tokens=264442, ETTS=413441, wrong-turn tax=219561, owner convergence=1.00

## Strongest positive signal
- ISSUE-04-entitlement-recovery-replay (proof-full_vs_bare): token delta 4.40%
## Strongest negative signal
- ISSUE-01-entitlement-grace (proof-full_vs_bare): token delta 24.53%

## Safe for abstract-level claims? yes
