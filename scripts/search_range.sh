#!/bin/sh
set -eu

if [ "$#" -lt 2 ]; then
  echo "usage: $0 START END [additional search options]" >&2
  exit 2
fi

start=$1
end=$2
shift 2
crate_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
output_dir="$crate_dir/results/$start-$end"

cd "$crate_dir"
RUSTFLAGS="-C target-cpu=native" cargo build --release
exec "$crate_dir/target/release/lerch-prime-search" search \
  --start "$start" \
  --end "$end" \
  --output-dir "$output_dir" \
  --resume \
  "$@"

