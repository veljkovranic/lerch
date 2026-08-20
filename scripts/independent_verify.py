#!/usr/bin/env python3
"""Independent Lerch verification using only Python bigint arithmetic."""

from __future__ import annotations

import argparse
import json
import math
import platform
import time


def is_prime_trial_division(n: int) -> bool:
    if n < 2:
        return False
    if n % 2 == 0:
        return n == 2
    for divisor in range(3, math.isqrt(n) + 1, 2):
        if n % divisor == 0:
            return False
    return True


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("prime", type=int)
    args = parser.parse_args()
    p = args.prime
    if not is_prime_trial_division(p):
        raise SystemExit(f"{p} is not prime")

    started = time.time()
    p2 = p * p
    p3 = p2 * p
    exponent = p - 1
    q1_mod_p = 0
    q2_mod_p = 0
    q1_mod_p2 = 0
    power_sum_mod_p3 = 0

    for a in range(1, p):
        power = pow(a, exponent, p3)
        q = (power - 1) // p
        q_mod_p = q % p
        q1_mod_p = (q1_mod_p + q_mod_p) % p
        q2_mod_p = (q2_mod_p + q_mod_p * q_mod_p) % p
        q1_mod_p2 = (q1_mod_p2 + q) % p2
        power_sum_mod_p3 = (power_sum_mod_p3 + power) % p3

    factorial_mod_p3 = 1
    for a in range(2, p):
        factorial_mod_p3 = (factorial_mod_p3 * a) % p3
    wilson_mod_p2 = ((factorial_mod_p3 + 1) // p) % p2
    q1_minus_wilson = (q1_mod_p2 - wilson_mod_p2) % p2
    power_sum_residue = (
        power_sum_mod_p3 - factorial_mod_p3 - p
    ) % p3

    print(
        json.dumps(
            {
                "p": p,
                "prime_by_trial_division": True,
                "implementation": platform.python_implementation(),
                "python": platform.python_version(),
                "q1_mod_p": q1_mod_p,
                "q2_mod_p": q2_mod_p,
                "q1_minus_wilson_mod_p2": str(q1_minus_wilson),
                "power_sum_minus_factorial_minus_p_mod_p3": str(
                    power_sum_residue
                ),
                "verified": q1_minus_wilson == 0
                and power_sum_residue == 0,
                "elapsed_seconds": time.time() - started,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()

