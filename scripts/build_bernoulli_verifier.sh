#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
output="$repo_root/target/bernoulli_verify"
compiler="${CC:-cc}"

mkdir -p "$repo_root/target"

if [[ -n "${FLINT_PREFIX:-}" ]]; then
    flint_prefix="$FLINT_PREFIX"
elif command -v brew >/dev/null 2>&1 && brew --prefix flint >/dev/null 2>&1 \
    && [[ -d "$(brew --prefix flint)/include/flint" ]]; then
    flint_prefix="$(brew --prefix flint)"
else
    flint_prefix=""
fi

if [[ -n "$flint_prefix" ]]; then
    "$compiler" -O3 -std=c11 -Wall -Wextra -Wpedantic \
        -I"$flint_prefix/include" -L"$flint_prefix/lib" \
        "$repo_root/tools/bernoulli_verify.c" \
        -o "$output" -lflint -lm -pthread
elif command -v pkg-config >/dev/null 2>&1 && pkg-config --exists flint; then
    # pkg-config output is intentionally word-split into compiler arguments.
    # shellcheck disable=SC2046
    "$compiler" -O3 -std=c11 -Wall -Wextra -Wpedantic \
        $(pkg-config --cflags flint) "$repo_root/tools/bernoulli_verify.c" \
        -o "$output" -lflint $(pkg-config --libs flint) -lm -pthread
else
    "$compiler" -O3 -std=c11 -Wall -Wextra -Wpedantic \
        "$repo_root/tools/bernoulli_verify.c" \
        -o "$output" -lflint -lm -pthread
fi

echo "built $output"
