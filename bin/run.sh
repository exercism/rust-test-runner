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
if [ ! -e Cargo.toml ]; then
    echo "WARNING: student did not upload Cargo.toml. This may cause build errors." | tee -a "$output_path/results.out"
elif [ ! -e Cargo.lock ]; then
    echo "WARNING: student did not upload Cargo.lock. This may cause build errors." | tee -a "$output_path/results.out"
fi

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

if grep -q "probable build failure" "$output_path"/results.json; then
   jq -n --rawfile m "$output_path"/results.out '{status: "error", message:$m}' > "$output_path"/results.json
fi
