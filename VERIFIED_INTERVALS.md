# Exhaustively verified intervals

This ledger contains only completed runs whose machine-readable manifest is
retained. Together, the production runs cover every prime through
$200{,}000{,}000$.

For research claims, treat "manifest.json" as authoritative and archive it
with the source revision, compiler version, CPU information, and verification
transcripts.

## Why the search used irregular rounds

The interval boundaries were inherited from earlier computations rather than
chosen as equal-sized search campaigns. In Section 2.3 of
[Sondow's paper](https://arxiv.org/pdf/1110.3113), Sondow reports that Marek
Wolf used a Mathematica implementation of the defining congruence to find no
additional Lerch primes in

- $1{,}000{,}003\le p\le4{,}496{,}113$,
- $18{,}816{,}869\le p\le18{,}977{,}773$, and
- $32{,}452{,}867\le p\le32{,}602{,}373$.

Sondow records that Wolf's computation took six months of CPU time on a
64-bit 2.7 GHz AMD Opteron. Together with the earlier search below
$1{,}000{,}003$, Wolf's work left two finite gaps in which a fifth Lerch prime
could occur:

$$
4{,}496{,}113<p<18{,}816{,}869
$$

and

$$
18{,}977{,}773<p<32{,}452{,}867,
$$

followed by the unbounded range $p>32{,}602{,}373$. The first search rounds
targeted the two historical gaps and continued to $50{,}000{,}000$. After the
new value was found, the remaining earlier ranges were searched as well. The
run through $4{,}496{,}112$, together with the adjacent historical-gap run,
reproduces Wolf's first exclusion. The short run from $18{,}816{,}870$ through
$18{,}977{,}772$ reproduces the interior of his second exclusion, whose
endpoints were already covered by the neighboring runs. The final production
run includes all of Wolf's third exclusion.

Thus the apparently irregular rounds, followed by the later extension, now
form one complete chain from $2$ through $200{,}000{,}000$, reproduce all three
results attributed to Wolf, and remove any dependence on the earlier
computation for the claim below. The shared boundaries at $32{,}452{,}867$ and
$50{,}000{,}000$ are intentional.

| Inclusive interval | Primes | Status | Manifest aggregate SHA-256 | Purpose |
|---|---:|---|---|---|
| 2–4,496,112 | 315,699 | complete | b0f38644d6e703b04e37e96e1a8863b893e12937600a6cb2220b9d1207c0350f | Reproduced the four established Lerch primes and Wolf's first exclusion |
| 4,496,113–18,816,869 | 884,401 | complete | e8c9ff61b15d486c64ac64749b7a96b19c08a9ede67d48df827e1a2448f49de4 | First historical gap; no Lerch hits |
| 18,816,870–18,977,772 | 9,599 | complete | cfc116fc790a54b7c98c6d0f03832100aa08171c80c2411cb9bdad4e8d210b5e | Reproduced the interior of Wolf's second exclusion; no Lerch hits |
| 18,977,773–32,452,867 | 790,302 | complete | e95d8a0be5fe8353eb5f42758f22a6337750f5c1782f7f15d65154d248d24e74 | Second historical gap; no Lerch hits |
| 32,452,867–50,000,000 | 1,001,134 | complete | 0ab143e08a3fafb32faba763e5e95a0d03777093dfb34d54c7bbc2f5ed6258ce | Reproduced Wolf's third exclusion, then found and verified Lerch prime 42,447,347 |
| 50,000,000–200,000,000 | 8,077,803 | complete | 769b0244a0bcb6027ad737dcc81b69bad3f876ee865a19f4ab0fe620afd0add3 | Extended the exhaustive search; no Lerch hits |

The complete chain contains exactly the Lerch hits
$3,103,839,2237,42{,}447{,}347$. It therefore establishes
$42{,}447{,}347$ as the fifth Lerch prime in increasing order.

Compact copies of all six production manifests, plus the earlier development
smoke manifest, are under "evidence/manifests/". The complete result trees are
in the release archives. Separately, the definition-level validation covered
all 9,591 odd primes below 100,000; its transcript is
"evidence/validation/validation_100k.txt". That validation is not listed as a
production search interval because it did not emit a search manifest.
