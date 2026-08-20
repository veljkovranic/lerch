# Reproducing the search evidence

This repository separates compact review evidence from the complete raw
checkpoint data. No result from the ongoing 50,000,000--200,000,000 campaign
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

The quick verification command for the reported candidate is:

~~~sh
target/release/lerch-prime-search verify --prime 42447347
~~~

The deliberately slow independent Python implementation can be run with:

~~~sh
python3 scripts/independent_verify.py 42447347
~~~

It performs trial-division primality testing and recomputes the Fermat
quotients, power sum, and factorial without calling the Rust implementation.

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
scripts/audit_search_results.py results/smoke_2_5000
scripts/audit_search_results.py results/4496113-18816869
scripts/audit_search_results.py results/18977773-32452867
scripts/audit_search_results.py results/32452867-50000000
~~~

The `manifest_sha256` JSON field is the SHA-256 of the colon-separated ordered
segment-result hashes. It is an aggregate commitment, not the byte-level hash
of `manifest.json`. Byte-level file hashes are recorded separately in
`SHA256SUMS`.

## Discovery evidence

`evidence/42447347/` contains the candidate segment, Rust verification JSON,
fresh-build transcript, and independent Python transcript. The complete
manifest is retained under `evidence/manifests/32452867-50000000.json` and is
also present inside the raw archive.
