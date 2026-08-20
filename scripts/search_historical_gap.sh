#!/bin/sh
set -eu

crate_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
exec "$crate_dir/scripts/search_range.sh" \
  4496113 18816869 \
  --chunk-size 100000 \
  "$@"

