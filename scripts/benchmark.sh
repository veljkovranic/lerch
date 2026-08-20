#!/bin/sh
set -eu

crate_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$crate_dir"
RUSTFLAGS="-C target-cpu=native" cargo run --release -- \
  benchmark --primes 10007,100003,1000003,5000011,10000019 --direct-max 20000

