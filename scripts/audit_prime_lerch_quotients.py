#!/usr/bin/env python3
"""Audit hashes, coverage, candidates, and summary totals for a completed run."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
from typing import Any


FORMAT_VERSION = "lerch-prime-quotient-search-v1"


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def digest_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def digest_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def is_prime(n: int) -> bool:
    if n < 2:
        return False
    if n % 2 == 0:
        return n == 2
    for divisor in range(3, math.isqrt(n) + 1, 2):
        if n % divisor == 0:
            return False
    return True


def expected_primes(start: int, end: int, limit: int) -> list[int]:
    values = [p for p in range(max(3, start | 1), end + 1, 2) if is_prime(p)]
    return values[:limit] if limit else values


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("result_dir", type=Path)
    args = parser.parse_args()
    root = args.result_dir.resolve()
    manifest = json.loads((root / "manifest.json").read_text())
    if manifest.get("version") != FORMAT_VERSION or manifest.get("status") != "complete":
        raise SystemExit("manifest is not a complete supported run")

    configuration = manifest["configuration"]
    configuration_sha256 = digest_bytes(canonical_bytes(configuration))
    if configuration_sha256 != manifest.get("configuration_sha256"):
        raise SystemExit("configuration SHA-256 mismatch")

    expected = expected_primes(
        configuration["start_prime"],
        configuration["end_prime"],
        configuration.get("limit", 0),
    )
    results: list[dict[str, Any]] = []
    for relative in manifest["result_files"]:
        path = root / relative
        result = json.loads(path.read_text())
        unhashed = {key: value for key, value in result.items() if key != "result_sha256"}
        if digest_bytes(canonical_bytes(unhashed)) != result.get("result_sha256"):
            raise SystemExit(f"result SHA-256 mismatch: {relative}")
        if result.get("configuration_sha256") != configuration_sha256:
            raise SystemExit(f"configuration mismatch: {relative}")
        if result.get("version") != FORMAT_VERSION:
            raise SystemExit(f"format mismatch: {relative}")
        if result["status"] == "composite_factor":
            factor = result["evidence"]
            if not is_prime(factor) or factor > configuration["trial_bound"]:
                raise SystemExit(f"invalid recorded factor for p={result['p']}")
            inline = result.get("inline_lerch_quotient")
            if inline is not None and inline % factor:
                raise SystemExit(f"recorded factor does not divide inline quotient for p={result['p']}")
        candidate = result.get("candidate")
        if result["status"] == "probable_prime":
            if not isinstance(candidate, dict):
                raise SystemExit(f"missing candidate metadata for p={result['p']}")
            candidate_path = root / candidate["path"]
            if (
                not candidate_path.is_file()
                or candidate_path.stat().st_size != candidate["bytes"]
                or digest_file(candidate_path) != candidate["sha256"]
            ):
                raise SystemExit(f"candidate artifact mismatch for p={result['p']}")
        elif candidate is not None:
            raise SystemExit(f"unexpected candidate artifact metadata for p={result['p']}")
        results.append(result)

    actual = [result["p"] for result in results]
    if actual != expected:
        missing = sorted(set(expected) - set(actual))
        extra = sorted(set(actual) - set(expected))
        raise SystemExit(f"prime coverage mismatch; missing={missing[:10]} extra={extra[:10]}")
    if manifest["prime_count"] != len(results):
        raise SystemExit("manifest prime count mismatch")

    counts: dict[str, int] = {}
    for result in results:
        counts[result["status"]] = counts.get(result["status"], 0) + 1
    if counts != manifest["status_counts"]:
        raise SystemExit("manifest status counts mismatch")
    prime_inputs = [
        result["p"]
        for result in results
        if result["status"] in {"prime_proven", "probable_prime"}
    ]
    if prime_inputs != manifest["prime_quotient_inputs"]:
        raise SystemExit("manifest prime-valued input list mismatch")

    aggregate = ":".join(result["result_sha256"] for result in results)
    if digest_bytes(aggregate.encode()) != manifest["manifest_sha256"]:
        raise SystemExit("manifest aggregate SHA-256 mismatch")

    print(
        json.dumps(
            {
                "audit": "passed",
                "prime_count": len(results),
                "first_prime": actual[0] if actual else None,
                "last_prime": actual[-1] if actual else None,
                "status_counts": counts,
                "prime_quotient_inputs": prime_inputs,
                "maximum_decimal_digits": max(
                    (result["decimal_digits"] for result in results), default=None
                ),
                "configuration_sha256": configuration_sha256,
                "manifest_sha256": manifest["manifest_sha256"],
            },
            indent=2,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

