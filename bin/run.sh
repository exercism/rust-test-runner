#!/usr/bin/env bash

set -e

export RUSTUP_HOME=/usr/local/rustup
export CARGO_HOME=/usr/local/cargo
export PATH=/usr/local/cargo/bin:$PATH

slug="$1"
solution_path="$2"
output_path="$3"

if [ -z "$output_path" ]; then
    output_path="$solution_path"
fi

# canonicalize output path
output_path="$(readlink -m "$output_path")"
rm -f "$output_path"/results.out  # we only ever append, so let's reset this

echo "slug:          $slug"
echo "solution path: $solution_path"
echo "output path:   $output_path"

if [ -z "$slug" ] || [ -z "$solution_path" ]; then
    echo "slug and solution path must be present"
    exit 1
fi

cd "$solution_path"
if [ -e Cargo.lock ]; then
    if [ "$(grep -c '\[\[package\]\]' Cargo.lock)" -gt 1 ]; then
        echo "{\"status\":\"error\",\"message\":\"building $slug: external crates not supported\",\"tests\":[]}" > "$output_path"/results.json
        exit
    fi
else
    echo "WARNING: student did not upload Cargo.lock. This may cause build errors." | tee -a "$output_path/results.out"
fi

RUST_BACKTRACE=1 \
RUST_TEST_TASKS=1 \
    cargo +nightly test \
    --offline \
    -- \
    -Z unstable-options \
    --include-ignored \
    --format json \
    2> >(tee -a "$output_path"/results.out >&2) \
    |\
        /opt/test-runner/bin/transform-output \
        > "$output_path"/results.json

