#!/usr/bin/env python3
"""Render public repository advisory score tables for the Jankurai paper."""

from __future__ import annotations

import argparse
import json
import statistics
from collections import Counter
from pathlib import Path
from typing import Any


REQUIRED_TOP_LEVEL_KEYS = {
    "run_root",
    "generated_at",
    "jankurai_version",
    "repo_count",
    "successful",
    "failed",
    "rows",
}

REQUIRED_ROW_KEYS = {
    "rank",
    "repo",
    "score",
    "finding_count",
    "hard_findings",
    "status",
    "weak_dimensions",
}

DIMENSION_LABELS = {
    "Jankurai tool adoption and CI replacement": "audit CI replacement",
    "Code shape and semantic surface": "code shape/semantic surface",
    "Context economy and agent instructions": "context routing",
    "Python containment and polyglot hygiene": "Python/\\allowbreak{}polyglot hygiene",
    "Proof lanes and test routing": "proof routing",
    "Security and supply-chain posture": "security/supply chain",
    "Build speed signals": "build speed",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render deterministic public repository score tables."
    )
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    return parser.parse_args()


def load_source(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise SystemExit(f"{path} must contain a JSON object")

    missing = REQUIRED_TOP_LEVEL_KEYS - set(data)
    if missing:
        raise SystemExit(f"{path} is missing top-level keys: {sorted(missing)}")

    rows = data["rows"]
    if not isinstance(rows, list):
        raise SystemExit(f"{path} rows must be a list")
    if len(rows) != data["repo_count"]:
        raise SystemExit(
            f"{path} repo_count={data['repo_count']} but rows={len(rows)}"
        )

    for index, row in enumerate(rows, start=1):
        if not isinstance(row, dict):
            raise SystemExit(f"{path} row {index} must be an object")
        missing_row = REQUIRED_ROW_KEYS - set(row)
        if missing_row:
            raise SystemExit(
                f"{path} row {index} is missing keys: {sorted(missing_row)}"
            )
        if not isinstance(row["weak_dimensions"], list):
            raise SystemExit(f"{path} row {index} weak_dimensions must be a list")

    return data


def tex_escape(value: str) -> str:
    replacements = {
        "\\": r"\textbackslash{}",
        "&": r"\&",
        "%": r"\%",
        "$": r"\$",
        "#": r"\#",
        "_": r"\_",
        "{": r"\{",
        "}": r"\}",
        "~": r"\textasciitilde{}",
        "^": r"\textasciicircum{}",
    }
    return "".join(replacements.get(char, char) for char in value)


def fmt_int(value: int) -> str:
    return f"{value:,}"


def fmt_float(value: float, digits: int = 1) -> str:
    return f"{value:.{digits}f}"


def score_ranked_rows(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return sorted(
        rows,
        key=lambda row: (
            -int(row["score"]),
            int(row["finding_count"]),
            row["repo"].lower(),
        ),
    )


def dimension_labels(row: dict[str, Any]) -> str:
    labels: list[str] = []
    for item in row["weak_dimensions"]:
        if not isinstance(item, dict):
            continue
        name = item.get("name")
        if not isinstance(name, str):
            continue
        labels.append(DIMENSION_LABELS.get(name, tex_escape(name)))
    return "; ".join(labels) if labels else "none reported"


def repo_path(repo: str) -> str:
    return f"\\path{{{repo}}}"


def score_cell(score: int, *, best_observed: bool = False) -> str:
    if best_observed:
        return f"\\JKScoreCellBest{{{score}}}"
    return f"\\JKScoreCell{{{score}}}"


def aggregate_rows(data: dict[str, Any]) -> list[tuple[str, str]]:
    rows = data["rows"]
    scores = sorted(int(row["score"]) for row in rows)
    findings_total = sum(int(row["finding_count"]) for row in rows)
    hard_total = sum(int(row["hard_findings"]) for row in rows)
    hard_share = (hard_total / findings_total * 100.0) if findings_total else 0.0
    upper_middle = scores[len(scores) // 2]
    return [
        ("Repositories scanned", fmt_int(int(data["repo_count"]))),
        ("Successful scans", fmt_int(int(data["successful"]))),
        ("Failed scans", fmt_int(int(data["failed"]))),
        ("Minimum score", fmt_int(min(scores))),
        ("Maximum score", f"{fmt_int(max(scores))} (best observed)"),
        ("Average score", fmt_float(statistics.fmean(scores))),
        ("Median score (upper middle)", fmt_int(upper_middle)),
        ("Findings total", fmt_int(findings_total)),
        ("Hard findings total", fmt_int(hard_total)),
        ("Hard findings share", f"{fmt_float(hard_share)}\\%"),
    ]


def weak_dimension_rows(rows: list[dict[str, Any]]) -> list[tuple[str, int]]:
    counter: Counter[str] = Counter()
    for row in rows:
        for item in row["weak_dimensions"]:
            if isinstance(item, dict) and isinstance(item.get("name"), str):
                counter[item["name"]] += 1
    ordered_names = [
        "Jankurai tool adoption and CI replacement",
        "Code shape and semantic surface",
        "Context economy and agent instructions",
        "Python containment and polyglot hygiene",
        "Proof lanes and test routing",
    ]
    return [(name, counter[name]) for name in ordered_names]


def render_top_table(rows: list[dict[str, Any]]) -> str:
    ranked = score_ranked_rows(rows)
    lines = [
        r"\newcommand{\PublicRepoScoreTopTable}{%",
        r"\begin{table*}[t]",
        r"\caption{Top observed advisory public-repository scores. The top score is best observed in this run, not a certification threshold.}",
        r"\label{tab:public-repo-top-scores}",
        r"\centering",
        r"\scriptsize",
        r"\setlength{\tabcolsep}{3pt}",
        r"\begin{tabularx}{\textwidth}{R{0.045\textwidth} L{0.245\textwidth} R{0.105\textwidth} R{0.075\textwidth} R{0.065\textwidth} Y}",
        r"\toprule",
        r"\JKTableHeader",
        r"\textbf{Rank} & \textbf{Repo} & \textbf{Score} & \textbf{Findings} & \textbf{Hard} & \textbf{Dominant gaps} \\",
        r"\midrule",
        r"\JKDenseRows",
    ]
    for rank, row in enumerate(ranked[:10], start=1):
        lines.append(
            " & ".join(
                [
                    str(rank),
                    repo_path(row["repo"]),
                    score_cell(int(row["score"]), best_observed=(rank == 1)),
                    fmt_int(int(row["finding_count"])),
                    fmt_int(int(row["hard_findings"])),
                    dimension_labels(row),
                ]
            )
            + r" \\"
        )
    lines.extend(
        [
            r"\bottomrule",
            r"\end{tabularx}",
            r"\JKResetRows",
            r"\end{table*}",
            r"}",
        ]
    )
    return "\n".join(lines)


def render_aggregate_table(data: dict[str, Any]) -> str:
    lines = [
        r"\newcommand{\PublicRepoScoreAggregateTable}{%",
        r"\begin{table}[t]",
        r"\caption{Aggregate advisory scan posture.}",
        r"\label{tab:public-repo-aggregate}",
        r"\centering",
        r"\scriptsize",
        r"\setlength{\tabcolsep}{4pt}",
        r"\begin{tabularx}{\columnwidth}{L{0.60\columnwidth} R{0.28\columnwidth}}",
        r"\toprule",
        r"\JKTableHeader",
        r"\textbf{Metric} & \textbf{Value} \\",
        r"\midrule",
        r"\JKDenseRows",
    ]
    for label, value in aggregate_rows(data):
        lines.append(f"{tex_escape(label)} & {value} " + r"\\")
    lines.extend(
        [
            r"\bottomrule",
            r"\end{tabularx}",
            r"\JKResetRows",
            r"\end{table}",
            r"}",
        ]
    )
    return "\n".join(lines)


def render_weak_dimension_table(rows: list[dict[str, Any]]) -> str:
    lines = [
        r"\newcommand{\PublicRepoWeakDimensionTable}{%",
        r"\begin{table}[t]",
        r"\caption{Most frequent weak-dimension sets in the advisory scan.}",
        r"\label{tab:public-repo-weak-dimensions}",
        r"\centering",
        r"\scriptsize",
        r"\setlength{\tabcolsep}{4pt}",
        r"\begin{tabularx}{\columnwidth}{Y R{0.18\columnwidth}}",
        r"\toprule",
        r"\JKTableHeader",
        r"\textbf{Weak dimension} & \textbf{Repos} \\",
        r"\midrule",
        r"\JKDenseRows",
    ]
    for label, count in weak_dimension_rows(rows):
        lines.append(f"{tex_escape(label)} & {count}/30 " + r"\\")
    lines.extend(
        [
            r"\bottomrule",
            r"\end{tabularx}",
            r"\JKResetRows",
            r"\end{table}",
            r"}",
        ]
    )
    return "\n".join(lines)


def render_appendix_table(rows: list[dict[str, Any]]) -> str:
    ranked = score_ranked_rows(rows)
    lines = [
        r"\newcommand{\PublicRepoScoreAppendixTable}{%",
        r"\begingroup",
        r"\scriptsize",
        r"\setlength{\tabcolsep}{3pt}",
        r"\setlength{\LTleft}{0pt}",
        r"\setlength{\LTright}{0pt}",
        r"\JKDenseRows",
        r"\begin{longtable}{@{}R{0.040\textwidth} R{0.045\textwidth} L{0.235\textwidth} R{0.055\textwidth} R{0.070\textwidth} R{0.065\textwidth} L{0.355\textwidth}@{}}",
        r"\caption{Full 30-repository advisory scoring run.}\label{tab:public-repo-full}\\",
        r"\toprule",
        r"\JKTableHeader",
        r"\textbf{Rank} & \textbf{Run \#} & \textbf{Repo} & \textbf{Score} & \textbf{Findings} & \textbf{Hard} & \textbf{Dominant gaps} \\",
        r"\midrule",
        r"\endfirsthead",
        r"\toprule",
        r"\JKTableHeader",
        r"\textbf{Rank} & \textbf{Run \#} & \textbf{Repo} & \textbf{Score} & \textbf{Findings} & \textbf{Hard} & \textbf{Dominant gaps} \\",
        r"\midrule",
        r"\endhead",
    ]
    for rank, row in enumerate(ranked, start=1):
        lines.append(
            " & ".join(
                [
                    str(rank),
                    str(int(row["rank"])),
                    repo_path(row["repo"]),
                    score_cell(int(row["score"]), best_observed=(rank == 1)),
                    fmt_int(int(row["finding_count"])),
                    fmt_int(int(row["hard_findings"])),
                    dimension_labels(row),
                ]
            )
            + r" \\"
        )
    lines.extend(
        [
            r"\bottomrule",
            r"\end{longtable}",
            r"\JKResetRows",
            r"\endgroup",
            r"}",
        ]
    )
    return "\n".join(lines)


def render(data: dict[str, Any], source: Path, out: Path) -> str:
    source_posix = source.as_posix()
    command = (
        "python3 tools/render_public_repo_scores.py "
        f"--source {source_posix} --out {out.as_posix()}"
    )
    return "\n".join(
        [
            "% Generated by: tools/render_public_repo_scores.py",
            f"% Source: {source_posix}",
            f"% Command: {command}",
            "% DO NOT EDIT BY HAND.",
            f"% Run root: {data['run_root']}",
            f"% Generated at: {data['generated_at']}",
            f"% Jankurai version: {data['jankurai_version']}",
            "",
            render_top_table(data["rows"]),
            "",
            render_aggregate_table(data),
            "",
            render_weak_dimension_table(data["rows"]),
            "",
            render_appendix_table(data["rows"]),
            "",
        ]
    )


def main() -> None:
    args = parse_args()
    data = load_source(args.source)
    output = render(data, args.source, args.out)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(output, encoding="utf-8")


if __name__ == "__main__":
    main()
