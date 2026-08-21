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

An additional computation subsequently evaluated the exact reduced fraction

$$
B_{42{,}447{,}346}=\frac{N}{254{,}684{,}082}
                   =\frac{N}{6p}
$$

with FLINT 3.0.1's isolated exact Bernoulli-number implementation. This path
does not call the Rust search or either factorial verifier. On the 64-core AMD
EPYC 7702 server it took 1,872.322 seconds using 64 FLINT threads.

The numerator $N$ has exactly 271,466,759 decimal digits. It begins

    80236671537442808117241000110493150791644273149431383747136759740392889493216755

and ends

    67866768163387804379962421936106526547789724809354285213903394943836397376231661

Its SHA-256 is

    d662c38e8f7e727a60079f76c9c697abbabfdb6db47ced26d80b69d15ced23d9

Reducing the exact fraction independently gave the congruence above and
`expected residue match: YES`. A separate streaming audit then read the saved
259 MiB numerator back from disk, recomputed its SHA-256 and exact digit count,
and again obtained the same residue. The compact evidence is in
`evidence/42447347/bernoulli/`; the full numerator is identified by the hash
recorded there and is kept outside Git because of its size.
