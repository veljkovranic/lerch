#!/bin/sh
set -eu

crate_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$crate_dir"
RUSTFLAGS="-C target-cpu=native" cargo run --release -- \
  reducer-benchmark --prime 1000003 --iterations 10000000

