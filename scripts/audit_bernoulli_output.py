#!/usr/bin/env python3
"""Stream-audit a saved exact Bernoulli numerator without loading it in RAM."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path


CHUNK_DIGITS = 4096


def decimal_file_mod(path: Path, modulus: int) -> tuple[int, int, str]:
    residue = 0
    digit_count = 0
    sign = 1
    digest = hashlib.sha256()

    with path.open("rb") as source:
        chunk = source.read(CHUNK_DIGITS)
        first = True
        while chunk:
            digest.update(chunk)
            following = source.read(CHUNK_DIGITS)
            if first:
                if chunk.startswith(b"-"):
                    sign = -1
                    chunk = chunk[1:]
                elif chunk.startswith(b"+"):
                    chunk = chunk[1:]
                first = False

            if not following:
                stripped = chunk.rstrip(b"\r\n")
                if chunk[len(stripped) :] not in (b"", b"\n", b"\r\n"):
                    raise ValueError(f"invalid trailing bytes in {path}")
                chunk = stripped

            if not chunk or not chunk.isdigit():
                raise ValueError(f"invalid decimal data in {path}")
            residue = (residue * pow(10, len(chunk), modulus) + int(chunk)) % modulus
            digit_count += len(chunk)
            chunk = following

    if digit_count == 0:
        raise ValueError(f"no digits in {path}")
    return (sign * residue) % modulus, digit_count, digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("numerator", type=Path)
    parser.add_argument("denominator", type=Path)
    parser.add_argument("--prime", type=int, default=42_447_347)
    parser.add_argument(
        "--expected", type=int, default=49_628_251_800_410_944_737_487
    )
    args = parser.parse_args()

    p = args.prime
    modulus = p**3
    numerator_mod, digits, numerator_sha256 = decimal_file_mod(
        args.numerator, modulus
    )
    denominator = int(args.denominator.read_text(encoding="ascii"))

    if denominator % p != 0:
        raise ValueError("Bernoulli denominator is not divisible by p")
    denominator_without_p = denominator // p
    if denominator_without_p % p == 0:
        raise ValueError("Bernoulli denominator is divisible by p more than once")

    residue = numerator_mod * pow(denominator_without_p, -1, modulus) % modulus
    matched = residue == args.expected % modulus

    print(f"numerator_digits={digits}")
    print(f"numerator_sha256={numerator_sha256}")
    print(f"denominator={denominator}")
    print(f"modulus_p_cubed={modulus}")
    print(f"p_times_bernoulli_mod_p_cubed={residue}")
    print(f"expected_residue_match={'YES' if matched else 'NO'}")
    return 0 if matched else 3


if __name__ == "__main__":
    raise SystemExit(main())
