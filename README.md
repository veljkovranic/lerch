# Lerch prime search

This self-contained Rust crate implements the primitive-root recurrence for
Fermat quotients. It searches deterministic prime intervals, records Lerch and
related exceptional conditions, checkpoints each completed chunk, and
independently rechecks every rare hit.

## Computational result

The completed search found the Lerch-prime candidate

$$
\boxed{p=42{,}447{,}347}.
$$

The defining congruence was reproduced by the optimized recurrence, a separate
definition-level Rust verifier using arithmetic modulo $p^2$ and $p^3$, and
an independent CPython bigint implementation. See
"DISCOVERY_42447347.md" and "evidence/42447347/" for the exact residues and
retained transcripts. External reproduction is invited before the result is
described as independently verified by another researcher.

The exact completed intervals and negative results are listed in
"VERIFIED_INTERVALS.md". Full raw segment data is stored as the compact archive
"release-assets/completed-search-results-through-50000000.tar.zst"; see
"REPRODUCING.md" for extraction and audit commands.

## Mathematical path

For an odd prime $p$, let

$$
q_p(a)=(a^{p-1}-1)/p,\qquad Q_r=\sum_{a=1}^{p-1}q_p(a)^r\pmod p.
$$

If $g$ is a primitive root and $c_j=\langle g^j\rangle_p$, write
$gc_j=c_{j+1}+k_jp$, and put $u_j=q_p(c_j)$. With
$v_j=c_j^{-1}$, the implementation uses

$$
u_{j+1}=u_j+q_p(g)+k_jv_{j+1}\pmod p.
$$

The loop starts with $(c,v,u)=(1,1,0)$, accumulates the current value,
then advances. Thus every nonzero residue is counted once. It tests

$$
Q_2+Q_1^2-2Q_1=0\pmod p
$$

for the Lerch condition, $Q_2=0$ for Gy exceptions, $Q_1=0$ for
Wilson primes, and $Q_1=2$. Optional $Q_3,Q_4$ accumulation is enabled
with the "--q3 --q4" options. When $Q_2\ne0$, it stores
$k_p=1-2L_p/Q_2\pmod p$.

The optimized path uses u64 values and u128 only in general modular
multiplication. The accepted search ceiling is 4,000,000,000, which keeps
$p^2$, recurrence products, and accumulators inside documented bounds.

## Build and validate

Rust 1.89 or newer is sufficient.

~~~sh
cd lerch_prime_search
RUSTFLAGS="-C target-cpu=native" cargo build --release
cargo test --release
scripts/validate_100k.sh
~~~

The ordinary test suite compares every recurrence-generated value against a
separate modular-power definition below 1,000, verifies all moments, checks the
exact known exceptional lists below 5,000, and verifies known Lerch primes with
two bigint congruences. The last script performs the explicitly requested
definition-level comparison for every prime below 100,000; it is deliberately
not part of the quick default test suite.

## Search and resume

~~~sh
scripts/search_range.sh 2 100000 --chunk-size 10000 --q3 --q4 --sample-every 25
scripts/search_range.sh 4496113 18816869 --chunk-size 100000
~~~

The second command can also be launched as "scripts/search_historical_gap.sh".
Pass "--threads N" to control parallelism. Chunks have fixed numeric
boundaries; Rayon schedules them dynamically, which balances the dominant
work approximately by $\sum p$ while leaving the decomposition reproducible.

Each chunk is written atomically below "results/START-END/segments". A complete
file includes its exact inclusive interval, prime count, sum of processed
primes, recurrence-step count, hit lists, configuration SHA-256, and result
SHA-256. Resume accepts a chunk only when its configuration hash matches.
The top-level "manifest.json" is the machine-readable record of exactly which
intervals are complete. Rare-hit verification transcripts live below
"verifications/"; a chunk is not marked complete if verification fails.

At $O(\sum_{p\le x}p)$, a scan to $10^8$ is a major compute campaign, not
a sensible single-workstation smoke test. Start with measured chunks, use the
benchmark command, then assign non-overlapping intervals to machines.

## Independent verification

~~~sh
target/release/lerch-prime-search verify --prime 2237
~~~

The reference path computes every Fermat quotient by a separate exponentiation
modulo $p^2$ and computes the Wilson quotient from a factorial modulo $p^2$.
For a Lerch candidate, the bigint path additionally verifies

$$
Q_1-W_p=0\pmod {p^2}
$$

and

$$
\sum_{a=1}^{p-1}a^{p-1}-(p-1)!-p=0\pmod {p^3}.
$$

## Benchmark and statistics

~~~sh
scripts/benchmark.sh > BENCHMARK.csv
scripts/benchmark_reducers.sh > REDUCERS.csv
scripts/analyze.py results/2-100000
~~~

The benchmark reports recurrence time and nanoseconds per step. At manageable
sizes it also times Method A (one modular power modulo $p^2$ per residue) and
Method B (direct bigint power sum modulo $p^3$) and reports both speedups.
The reducer benchmark compares u128 remainder, Barrett, and Montgomery kernels
for one fixed prime; see "IMPLEMENTATION_NOTES.md" before extrapolating it to
the full recurrence.

Sampling is off by default. "--sample-every N" stores every Nth prime in each
chunk, including normalized $Q_1,Q_2,L_p,k_p$ inputs. The analysis script
writes "normalized_samples.csv" and "statistics.json" with a Lerch-residue
histogram, Pearson chi-square statistic, and $Q_1/Q_2$ correlation.

## File map

- "src/recurrence.rs": optimized constant-memory recurrence.
- "src/reduction.rs": tested Barrett and Montgomery experiment kernels.
- "src/reference.rs": slow definition-based implementation.
- "src/sieve.rs": segmented prime generation.
- "src/search.rs": parallel chunks, atomic checkpoints, manifests, hit checks.
- "src/verify.rs": independent rare-hit and bigint Lerch verification.
- "tests/correctness.rs": cross-method regression/property coverage.
- "IMPLEMENTATION_NOTES.md": optimization and batch/Kummer notes.
- "VERIFIED_INTERVALS.md": human-readable ledger of retained completed runs.
- "REPRODUCING.md": commands for rebuilding, verifying, extracting, and
  auditing the published evidence.
- "evidence/": compact manifests and discovery transcripts suitable for
  direct review in GitHub.
- "release-assets/": compressed complete result data intended to be attached
  to a tagged GitHub release.

## References

- Jonathan Sondow, "Lerch quotients, Lerch primes, Fermat-Wilson quotients,
  and the Wieferich-non-Wilson primes 2, 3, 14771," arXiv:1110.3113,
  https://arxiv.org/abs/1110.3113.
- John Blythe Dobson, "A note on Lerch primes," arXiv:1311.2242,
  https://arxiv.org/abs/1311.2242.
- OEIS Foundation, A197632, "Lerch primes,"
  https://oeis.org/A197632.
