#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
binary="$repo_root/target/bernoulli_verify"
test_dir="$(mktemp -d "${TMPDIR:-/tmp}/bernoulli-verifier-test.XXXXXX")"
prefix="$test_dir/B_4"

cleanup() {
    rm -f "$prefix.numerator.txt" "$prefix.denominator.txt" \
        "$prefix.summary.json" "$test_dir/audit.txt"
    rmdir "$test_dir"
}
trap cleanup EXIT

"$repo_root/scripts/build_bernoulli_verifier.sh"

# B_4 = -1/30, so 5*B_4 = -1/6 = 104 (mod 5^3).
"$binary" --prime 5 --expected 104 --progress-seconds 0 --write-prefix "$prefix"
"$repo_root/scripts/audit_bernoulli_output.py" \
    "$prefix.numerator.txt" "$prefix.denominator.txt" \
    --prime 5 --expected 104 | tee "$test_dir/audit.txt"
grep -q '^expected_residue_match=YES$' "$test_dir/audit.txt"

# B_6 = 1/42, so 7*B_6 = 1/6 = 286 (mod 7^3).
"$binary" --prime 7 --expected 286 --progress-seconds 0

echo "Bernoulli verifier tests passed"
