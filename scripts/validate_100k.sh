#!/bin/sh
set -eu

crate_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$crate_dir"
RUSTFLAGS="-C target-cpu=native" cargo run --release -- \
  validate --limit 100000 --bigint-limit 300

