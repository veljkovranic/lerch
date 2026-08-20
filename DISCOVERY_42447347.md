# Fifth Lerch prime: 42,447,347

The exhaustive search of every prime through 50,000,000 found exactly five
Lerch primes: 3, 103, 839, 2237, and 42,447,347. Thus the fifth Lerch prime in
increasing order is

$$
p=42,447,347.
$$

The complete 1,755-segment result set passed interval-continuity, configuration
hash, result hash, manifest hash, independently generated prime-count, prime
sum, and recurrence-step audits.

The optimized recurrence gave

$$
Q_1=34,227,565,\qquad Q_2=10,415,263\pmod p,
$$

and

$$
Q_2+Q_1^2-2Q_1=0\pmod p.
$$

The server-side independent verifier and a fresh local Rust build on a
different architecture both reproduced the direct Fermat-quotient values and
Wilson quotient. A third implementation using CPython's independent bigint
modular exponentiation and trial-division primality check reproduced the same
values. All three obtained

$$
Q_1-W_p=0\pmod {p^2}
$$

and

$$
\sum_{a=1}^{p-1}a^{p-1}-(p-1)!-p=0\pmod {p^3}.
$$

The manifest aggregate SHA-256 (the hash of the ordered segment-result hashes)
is

    0ab143e08a3fafb32faba763e5e95a0d03777093dfb34d54c7bbc2f5ed6258ce

The discovery segment result SHA-256 is

    5f8fbce99c813c98fe1a20943d8ec1fd01415c9b868b4344ee207cec297671f0

The source used for publication is identified by the tagged Git commit rather
than by an undocumented source-tree archive hash. The retained transcripts are:

- "evidence/42447347/p_42447347.json"
- "evidence/42447347/local_clean_verify_42447347.txt"
- "evidence/42447347/python_independent_verify_42447347.txt"
- "evidence/42447347/segment_00000999.json"

By the Bernoulli-number criterion, the defining congruence also implies

$$
42{,}447{,}347 B_{42{,}447{,}346}
\equiv 49{,}628{,}251{,}800{,}410{,}944{,}737{,}487
\pmod {42{,}447{,}347^3}.
$$

This residue is a consequence of the independently checked factorial
congruence; it is not presented as a separate Bernoulli-number computation.
