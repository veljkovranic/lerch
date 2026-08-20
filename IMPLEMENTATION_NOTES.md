# Implementation and research notes

## Baseline decisions

The recurrence dominates primitive-root discovery: a prime near \(p\) needs
\(p-1\) recurrence steps, while factoring \(p-1\) by the base primes costs at
most \(O(\sqrt p/\log p)\) trial divisions. Small primitive-root candidates are
tested first. This also keeps recurrence products cheap on typical inputs.

The hot loop uses native remainder on u64. A changing modulus per prime makes
Montgomery conversion unattractive for the recurrence, and \(p\le4\cdot10^9\)
makes all hot products fit in u64. General modular multiplication uses a u128
product for exponentiation and optional higher moments. The included reducer
microbenchmark found Barrett and encoded Montgomery multiplication faster than
a u128 remainder for a fixed modulus on the development host. The recurrence
predominantly uses u64 division and mixes additions with reductions;
conversion/setup costs and whole-loop measurements do not yet justify
replacing that path. The alternative reducers remain tested experimental
kernels rather than asserted wins.

Reductions are delayed only where bounds are obvious: Q1 uses one conditional
subtraction, while Q2 reduces each step. More aggressive delayed Q2
accumulation quickly exceeds u64 and forces u128, which is not assumed faster.
Loop unrolling is constrained by the sequential u, c, and v dependencies and
remains an experiment.

## Parallel work model

Fixed numeric chunks are reproducible and independently resumable. Dynamic
Rayon scheduling avoids static prime-count imbalance. For a narrow interval
near \(x\), equal-width chunks have nearly equal expected work
\(\sum p\approx x\,\Delta x/\log x\). Very wide campaigns should use smaller
chunks or precompute boundaries from the estimated integral of \(x/\log x\).

## GPU status

No GPU implementation is included. A thread per prime exposes parallelism but
has severe load imbalance and a long sequential dependency chain. A prefix
formulation for \(u_j\) is mathematically possible, but it must first generate
the multiplicative orbit and carry terms and then perform scan/reductions.
Memory traffic and setup may dominate. This is an experiment, not a presumed
speedup.

## Batch Kummer/Bernoulli direction

No credible near-linear batch algorithm is claimed here. Computing
\(D_p=(F_p(2)-F_p(1))/p\) requires one more p-adic digit than ordinary Kummer
congruences expose. Product/remainder trees can amortize reductions of a
shared integer, but the Bernoulli indices \(p-1\) and \(2p-2\) vary with every
prime, so there is no single fixed numerator to reduce. Harvey-style
multimodular Bernoulli methods may reduce constants or batch ranges of indices,
but turning that into near-linear total work over varying moduli needs a
careful new derivation. It should not replace the recurrence baseline without
an independently verified proof and benchmark.
