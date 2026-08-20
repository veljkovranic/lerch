# Lerch prime search

This self-contained Rust crate implements the primitive-root recurrence for
Fermat quotients. It searches deterministic prime intervals, records Lerch and
related exceptional conditions, checkpoints each completed chunk, and
independently rechecks every rare hit.

## Computational result

For an odd prime $p$, define the Fermat quotient and Wilson quotient by

$$
q_p(a)=\frac{a^{p-1}-1}{p},
\qquad
W_p=\frac{(p-1)!+1}{p}.
$$

Lerch's congruence implies that the Lerch quotient

$$
\ell_p=\frac{\displaystyle\sum_{a=1}^{p-1}q_p(a)-W_p}{p}
$$

is an integer. A **Lerch prime** is an odd prime $p$ for which
$p\mid\ell_p$. Equivalently,

$$
\sum_{a=1}^{p-1}a^{p-1}-(p-1)!-p\equiv0\pmod {p^3}.
$$

Before this computation, the published literature recorded only four Lerch
primes: $3$, $103$, $839$, and $2237$. Sondow reported these four through
$3\times10^6$ in [*Lerch Quotients, Lerch Primes, Fermat-Wilson Quotients,
and the Wieferich-non-Wilson Primes 2, 3, 14771*](https://arxiv.org/abs/1110.3113),
and Dobson subsequently listed the same four in
[*A note on Lerch primes*](https://arxiv.org/abs/1311.2242).

The completed intervals searched by this project found one new value:

$$
\boxed{p=42{,}447{,}347}.
$$

The project verified that this value is prime and satisfies the defining
congruence. The result was reproduced by the optimized recurrence, a separate
definition-level Rust verifier using arithmetic modulo $p^2$ and $p^3$, and an
independent CPython bigint implementation. See
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

## References

- Jonathan Sondow, "Lerch quotients, Lerch primes, Fermat-Wilson quotients,
  and the Wieferich-non-Wilson primes 2, 3, 14771," arXiv:1110.3113,
  https://arxiv.org/abs/1110.3113.
- John Blythe Dobson, "A note on Lerch primes," arXiv:1311.2242,
  https://arxiv.org/abs/1311.2242.
- OEIS Foundation, A197632, "Lerch primes,"
  https://oeis.org/A197632.
