#!/bin/sh
set -eu

crate_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
workers=${1:-64}

if ! command -v gp >/dev/null 2>&1; then
  echo "PARI/GP is required. On Ubuntu: sudo apt-get install pari-gp" >&2
  exit 1
fi

exec python3 "$crate_dir/scripts/search_prime_lerch_quotients.py" \
  --start-prime 3 \
  --end-prime 100003 \
  --workers "$workers" \
  --trial-bound 10000 \
  --target-digits 500000 \
  --output-dir "$crate_dir/results/prime-lerch-quotients-3-100003" \
  --resume
