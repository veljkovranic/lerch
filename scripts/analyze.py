#!/usr/bin/env python3
"""Extract samples and basic uniformity/correlation statistics from a search."""

from __future__ import annotations

import argparse
import csv
import json
import math
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("result_dir", type=Path)
    parser.add_argument("--bins", type=int, default=20)
    args = parser.parse_args()
    if args.bins < 2:
        parser.error("--bins must be at least 2")

    manifest = json.loads((args.result_dir / "manifest.json").read_text())
    rows = []
    for relative in manifest["segment_files"]:
        segment = json.loads((args.result_dir / relative).read_text())
        for record in segment["records"]:
            if not record["sample"]:
                continue
            inv = record["invariants"]
            p = inv["p"]
            rows.append(
                {
                    "p": p,
                    "q1_over_p": inv["q1"] / p,
                    "q2_over_p": inv["q2"] / p,
                    "lerch_over_p": (
                        inv["lerch_remainder"] / p
                        if inv["lerch_remainder"] is not None
                        else ""
                    ),
                    "k_over_p": (
                        inv["generalized_k"] / p
                        if inv["generalized_k"] is not None
                        else ""
                    ),
                }
            )

    csv_path = args.result_dir / "normalized_samples.csv"
    fieldnames = [
        "p", "q1_over_p", "q2_over_p", "lerch_over_p", "k_over_p"
    ]
    with csv_path.open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)

    histogram = [0] * args.bins
    for row in rows:
        if row["lerch_over_p"] == "":
            continue
        histogram[min(int(row["lerch_over_p"] * args.bins), args.bins - 1)] += 1
    histogram_count = sum(histogram)
    expected = histogram_count / args.bins if histogram_count else 0
    chi_square = (
        sum((count - expected) ** 2 / expected for count in histogram)
        if expected
        else None
    )

    def correlation(x_name: str, y_name: str) -> float | None:
        pairs = [(float(r[x_name]), float(r[y_name])) for r in rows]
        if len(pairs) < 2:
            return None
        mx = sum(x for x, _ in pairs) / len(pairs)
        my = sum(y for _, y in pairs) / len(pairs)
        covariance = sum((x - mx) * (y - my) for x, y in pairs)
        vx = sum((x - mx) ** 2 for x, _ in pairs)
        vy = sum((y - my) ** 2 for _, y in pairs)
        return covariance / math.sqrt(vx * vy) if vx and vy else None

    summary = {
        "sample_count": len(rows),
        "bins": args.bins,
        "lerch_histogram": histogram,
        "uniform_expected_per_bin": expected,
        "pearson_chi_square": chi_square,
        "q1_q2_pearson_correlation": correlation("q1_over_p", "q2_over_p"),
        "source_manifest_sha256": manifest["manifest_sha256"],
    }
    summary_path = args.result_dir / "statistics.json"
    summary_path.write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
