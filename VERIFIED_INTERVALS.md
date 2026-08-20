# Exhaustively verified intervals

This ledger contains only completed runs whose machine-readable manifest is
retained. The repository starts with no claimed large historical interval.

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

followed by the unbounded range $p>32{,}602{,}373$. The first two production
rounds below were designed to fill those two historical gaps. The third round
started at $32{,}452{,}867$, deliberately overlapping Wolf's last exclusion,
and then continued to $50{,}000{,}000$. That overlap removes dependence on an
external boundary convention, independently rechecks part of Wolf's result,
and gives this repository continuous retained coverage from $18{,}977{,}773$
through $50{,}000{,}000$. Inclusive endpoints and the shared boundary at
$32{,}452{,}867$ are intentional.

| Inclusive interval | Primes | Status | Manifest aggregate SHA-256 | Purpose |
|---|---:|---|---|---|
| 2–5,000 | 669 | complete | 458f3ff6658ca78a778683552329282018bb29012dd66e9c9355447dbfd2d238 | Development smoke search with Q3/Q4 and rare-hit verification |
| 4,496,113–18,816,869 | 884,401 | complete | e8c9ff61b15d486c64ac64749b7a96b19c08a9ede67d48df827e1a2448f49de4 | First historical gap; no Lerch hits |
| 18,977,773–32,452,867 | 790,302 | complete | e95d8a0be5fe8353eb5f42758f22a6337750f5c1782f7f15d65154d248d24e74 | Second historical gap; no Lerch hits |
| 32,452,867–50,000,000 | 1,001,134 | complete | 0ab143e08a3fafb32faba763e5e95a0d03777093dfb34d54c7bbc2f5ed6258ce | Found and independently verified Lerch prime 42,447,347 |

Compact copies of all four manifests are under "evidence/manifests/" and the
complete result trees are in the release archive. Separately, the
definition-level validation covered all 9,591 odd primes below 100,000; its
transcript is "evidence/validation/validation_100k.txt". That validation is not listed as
a production search interval because it did not emit a search manifest.
