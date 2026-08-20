# Baseline benchmark

Environment:

- Apple M1 Pro, macOS 15.6
- rustc 1.89.0
- release profile with native CPU features
- single-prime measurements; wall-clock timing

| p | root | steps | recurrence | ns/step | effective mult/s | direct Q | bigint sum | A/C | B/C |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 10,007 | 5 | 10,006 | 0.000077625 s | 7.758 | 516 M | 0.000778917 s | 0.033464083 s | 10.03× | 431.10× |
| 100,003 | 2 | 100,002 | 0.000609250 s | 6.092 | 657 M | — | — | — | — |
| 1,000,003 | 2 | 1,000,002 | 0.006366500 s | 6.366 | 628 M | — | — | — | — |
| 5,000,011 | 2 | 5,000,010 | 0.028342416 s | 5.668 | 706 M | — | — | — | — |
| 10,000,019 | 6 | 10,000,018 | 0.056644084 s | 5.664 | 706 M | — | — | — | — |

Method A is the separate modular exponentiation modulo \(p^2\) for every
nonzero residue. Method B is the direct bigint power sum modulo \(p^3\).
Method C is the primitive-root recurrence. Direct methods are intentionally
skipped above the configured "--direct-max" bound.

"Effective mult/s" counts the four ordinary products in each recurrence step;
it is a throughput indicator, not a claim that all four are equivalent modular
multiplications.

The fixed-modulus reducer microbenchmark at \(p=1,000,003\), ten million
dependent multiplications, measured 8.629 ns for u128 remainder, 6.355 ns for
Barrett, and 6.393 ns for encoded Montgomery. These kernels are retained for
experimentation; this microbenchmark does not show that replacing the complete
mixed u64 recurrence loop would be faster.

These are short baseline timings rather than statistically rigorous
microbenchmarks. Rerun "scripts/benchmark.sh" on the actual search host and
retain the resulting CSV before making runtime projections.
