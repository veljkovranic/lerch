#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
output_dir="${1:-$repo_root/bernoulli-output/42447347}"
threads="${FLINT_THREADS:-1}"
binary="$repo_root/target/bernoulli_verify"
prefix="$output_dir/B_42447346"
transcript="$output_dir/run.txt"
checksums="$output_dir/SHA256SUMS"

if [[ -e "$output_dir" ]]; then
    echo "error: refusing to reuse existing output directory: $output_dir" >&2
    exit 1
fi

mkdir -p "$output_dir"
"$repo_root/scripts/build_bernoulli_verifier.sh"

{
    echo "started_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "hostname=$(hostname)"
    echo "uname=$(uname -a)"
    echo "threads_requested=$threads"
} | tee "$transcript"

"$binary" \
    --prime 42447347 \
    --expected 49628251800410944737487 \
    --threads "$threads" \
    --progress-seconds 60 \
    --write-prefix "$prefix" 2>&1 | tee -a "$transcript"

"$repo_root/scripts/audit_bernoulli_output.py" \
    "$prefix.numerator.txt" "$prefix.denominator.txt" \
    | tee "$output_dir/streaming_audit.txt"

echo "finished_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee -a "$transcript"

if command -v sha256sum >/dev/null 2>&1; then
    (cd "$output_dir" && sha256sum B_42447346.* run.txt streaming_audit.txt) > "$checksums"
else
    (cd "$output_dir" && shasum -a 256 B_42447346.* run.txt streaming_audit.txt) > "$checksums"
fi

echo "completed; outputs are in $output_dir"
