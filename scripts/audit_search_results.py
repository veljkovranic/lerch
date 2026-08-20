#!/usr/bin/env python3
"""Audit a completed lerch-prime-search result directory.

The default audit checks segment and manifest hashes, exact interval coverage,
aggregate totals and hit lists, referenced verification transcripts, and
independently regenerates the primes in every segment.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
from typing import Any


def fail(message: str) -> None:
    raise SystemExit(f"audit failed: {message}")


def sha256_json(value: Any) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":")
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def segment_hash(segment: dict[str, Any]) -> str:
    payload = [
        segment["start"],
        segment["end"],
        segment["prime_count"],
        segment["sum_of_p_processed"],
        segment["lerch_hits"],
        segment["q2_zero_hits"],
        segment["wilson_hits"],
        segment["q1_equals_2_hits"],
        segment["q3_zero_hits"],
        segment["q4_zero_hits"],
        segment["records"],
    ]
    return sha256_json(payload)


def base_primes(limit: int) -> list[int]:
    sieve = bytearray(b"\x01") * (limit + 1)
    if limit >= 0:
        sieve[0] = 0
    if limit >= 1:
        sieve[1] = 0
    for p in range(2, math.isqrt(limit) + 1):
        if sieve[p]:
            start = p * p
            sieve[start : limit + 1 : p] = b"\x00" * (
                (limit - start) // p + 1
            )
    return [p for p, is_prime in enumerate(sieve) if is_prime]


def interval_prime_totals(start: int, end: int, bases: list[int]) -> tuple[int, int]:
    flags = bytearray(b"\x01") * (end - start + 1)
    for p in bases:
        if p * p > end:
            break
        first = max(p * p, ((start + p - 1) // p) * p)
        if first <= end:
            flags[first - start : end - start + 1 : p] = b"\x00" * (
                (end - first) // p + 1
            )
    if start <= 1:
        for n in range(start, min(end, 1) + 1):
            flags[n - start] = 0
    primes = [start + offset for offset, flag in enumerate(flags) if flag]
    return len(primes), sum(primes)


def audit(directory: Path, regenerate_primes: bool) -> dict[str, Any]:
    manifest_path = directory / "manifest.json"
    if not manifest_path.is_file():
        fail(f"missing {manifest_path}")
    manifest = json.loads(manifest_path.read_text())
    if manifest.get("status") != "complete":
        fail("manifest is not complete")

    listed = manifest["segment_files"]
    intervals = manifest["completed_intervals"]
    if len(listed) != len(intervals) or not listed:
        fail("segment file and interval counts disagree or are empty")

    bases = (
        base_primes(math.isqrt(int(manifest["requested_end"])))
        if regenerate_primes
        else []
    )
    segments: list[dict[str, Any]] = []
    expected_start = int(manifest["requested_start"])
    for index, relative_name in enumerate(listed):
        path = directory / relative_name
        if not path.is_file():
            fail(f"missing segment {relative_name}")
        segment = json.loads(path.read_text())
        if segment.get("status") != "complete":
            fail(f"incomplete segment {relative_name}")
        if segment_hash(segment) != segment.get("result_sha256"):
            fail(f"result hash mismatch in {relative_name}")
        if [segment["start"], segment["end"]] != intervals[index]:
            fail(f"manifest interval mismatch for {relative_name}")
        if segment["start"] != expected_start:
            fail(f"coverage gap or overlap before {relative_name}")
        expected_start = segment["end"] + 1

        if regenerate_primes:
            count, total = interval_prime_totals(
                int(segment["start"]), int(segment["end"]), bases
            )
            if count != segment["prime_count"]:
                fail(f"prime-count mismatch in {relative_name}")
            if str(total) != segment["sum_of_p_processed"]:
                fail(f"prime-sum mismatch in {relative_name}")
            if str(total - count) != segment["recurrence_steps"]:
                fail(f"recurrence-step mismatch in {relative_name}")

        for record in segment["records"]:
            verification_name = record.get("verification_file")
            if verification_name is not None:
                verification_path = directory / verification_name
                if not verification_path.is_file():
                    fail(f"missing verification {verification_name}")
                verification = json.loads(verification_path.read_text())
                if not verification.get("verified"):
                    fail(f"failed verification {verification_name}")
                if verification.get("p") != record["invariants"]["p"]:
                    fail(f"verification prime mismatch in {verification_name}")
        segments.append(segment)

    if expected_start - 1 != int(manifest["requested_end"]):
        fail("coverage does not reach requested endpoint")

    aggregate_hash = hashlib.sha256(
        ":".join(segment["result_sha256"] for segment in segments).encode()
    ).hexdigest()
    if aggregate_hash != manifest.get("manifest_sha256"):
        fail("manifest aggregate hash mismatch")

    scalar_totals = {
        "prime_count": sum(segment["prime_count"] for segment in segments),
        "sum_of_p_processed": str(
            sum(int(segment["sum_of_p_processed"]) for segment in segments)
        ),
    }
    for key, value in scalar_totals.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} mismatch")

    for key in (
        "lerch_hits",
        "q2_zero_hits",
        "wilson_hits",
        "q1_equals_2_hits",
        "q3_zero_hits",
        "q4_zero_hits",
    ):
        combined = [hit for segment in segments for hit in segment[key]]
        if combined != manifest.get(key):
            fail(f"manifest {key} mismatch")

    return {
        "audit": "passed",
        "directory": str(directory),
        "requested_interval": [
            manifest["requested_start"],
            manifest["requested_end"],
        ],
        "segments": len(segments),
        "prime_count": manifest["prime_count"],
        "lerch_hits": manifest["lerch_hits"],
        "manifest_aggregate_sha256": manifest["manifest_sha256"],
        "independent_prime_regeneration": regenerate_primes,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("result_directory", type=Path)
    parser.add_argument(
        "--skip-prime-regeneration",
        action="store_true",
        help="skip the independent segmented-sieve count and sum checks",
    )
    args = parser.parse_args()
    result = audit(args.result_directory, not args.skip_prime_regeneration)
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
