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
   # escape double quotes in message
   sed -i 's/"/\\"/g' "$output_path"/results.out
   # build json with message
   echo '{"status": "error", "message":"' > "$output_path"/results.json
   cat "$output_path"/results.out >> "$output_path"/results.json
   echo '"}' >> "$output_path"/results.json
   # Replace line endings with \n string
   # https://stackoverflow.com/questions/38672680/replace-newlines-with-literal-n/38674872
   sed -i -E ':a;N;$!ba;s/\r{0,1}\n/\\n/g' "$output_path"/results.json
   echo "Finished with error"
fi
