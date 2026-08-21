# Reproducing the search evidence

This repository separates compact review evidence from the complete raw
checkpoint data. No result from the separate 50,000,000--200,000,000 campaign
is included or claimed here.

## Environment used for the large search

- Ubuntu 24.04
- AMD EPYC 7702, 64 physical cores / 128 hardware threads
- 1 TiB RAM
- Rust 1.89.0
- release build with native CPU features

The algorithm is deterministic. Thread scheduling changes completion order and
elapsed times but not chunk boundaries, primes, residues, or result hashes.

## Build and run the test suite

~~~sh
RUSTFLAGS="-C target-cpu=native" cargo build --release
cargo test --release
scripts/validate_100k.sh
~~~

The quick verification command for the reported fifth Lerch prime is:

~~~sh
target/release/lerch-prime-search verify --prime 42447347
~~~

The deliberately slow independent Python implementation can be run with:

~~~sh
python3 scripts/independent_verify.py 42447347
~~~

It performs trial-division primality testing and recomputes the Fermat
quotients, power sum, and factorial without calling the Rust implementation.

## Independent exact Bernoulli-number verification

The standalone C verifier in `tools/bernoulli_verify.c` uses
[FLINT's isolated exact Bernoulli-number implementation](https://flintlib.org/doc/bernoulli.html).
For large even indices, the tested FLINT 3.0.1 and 3.6 implementations use
their multimodular algorithm. This computation does not call the Rust search
or the factorial-based Python verifier.

Install FLINT 3.x and build and test the wrapper:

~~~sh
# macOS
brew install flint

# Ubuntu/Debian
sudo apt-get install libflint-dev

scripts/test_bernoulli_verifier.sh
~~~

Inspect the size estimate without starting the computation:

~~~sh
scripts/build_bernoulli_verifier.sh
target/bernoulli_verify --estimate-only
~~~

The production helper refuses to reuse an existing output directory. It writes
the exact numerator and denominator, a JSON summary, a run transcript with a
60-second elapsed-time heartbeat, and SHA-256 checksums:

~~~sh
FLINT_THREADS=64 scripts/run_bernoulli_42447346.sh /data/bernoulli-42447346
~~~

FLINT's exact call is monolithic: the heartbeat shows that the process is
alive, but there is no checkpoint/resume facility within this computation.
Run it under `tmux` or an equivalent persistent session on the server. The
decisive line in the completed transcript must be:

~~~text
expected residue match: YES
~~~

The production helper then independently streams the saved numerator back from
disk and reduces it modulo \(p^3\). This determines the exact digit count and
confirms the SHA-256 hash without loading the whole integer into memory. FLINT's
`fmpz_sizeinbase` value in the primary transcript is an upper bound and can be
one digit larger than the exact count for a non-binary base.

To repeat that audit manually:

~~~sh
scripts/audit_bernoulli_output.py \
  /data/bernoulli-42447346/B_42447346.numerator.txt \
  /data/bernoulli-42447346/B_42447346.denominator.txt
~~~

This establishes the Bernoulli congruence by an implementation path separate
from the factorial identity used in the existing evidence. The production
output should only be added to the evidence set after the run completes and
its checksums have been copied and checked on another machine.

## Extract the complete result archive

Install `zstd`, then run:

~~~sh
zstd -dc release-assets/completed-search-results-through-50000000.tar.zst |
  tar -xf -
~~~

This creates `results/` with every original segment, manifest, rare-hit
verification, validation transcript, and benchmark CSV. The directory is
ignored by Git because the compressed archive is the canonical published
copy.

Verify the archive and evidence file checksums:

~~~sh
shasum -a 256 -c SHA256SUMS
~~~

## Audit completed intervals

The auditor recomputes the Rust-compatible hashes, checks exact interval
continuity and aggregate fields, validates every referenced rare-hit
transcript, and independently regenerates the primes in every segment with a
Python segmented sieve:

~~~sh
scripts/audit_search_results.py results/2-4496112
scripts/audit_search_results.py results/4496113-18816869
scripts/audit_search_results.py results/18816870-18977772
scripts/audit_search_results.py results/18977773-32452867
scripts/audit_search_results.py results/32452867-50000000
~~~

These five manifests form complete coverage through 50,000,000. Taken
together at their shared boundaries, they reproduce the three negative
intervals computed by Marek Wolf and reported in Section 2.3 of
[Sondow's paper](https://arxiv.org/pdf/1110.3113).

The `manifest_sha256` JSON field is the SHA-256 of the colon-separated ordered
segment-result hashes. It is an aggregate commitment, not the byte-level hash
of `manifest.json`. Byte-level file hashes are recorded separately in
`SHA256SUMS`.

## Discovery evidence

`evidence/42447347/` contains the discovery segment, Rust verification JSON,
fresh-build transcript, and independent Python transcript. The complete
manifest is retained under `evidence/manifests/32452867-50000000.json` and is
also present inside the raw archive.
