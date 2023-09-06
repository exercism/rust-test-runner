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
fi

release=""
if jq --exit-status '.custom?."test-in-release-mode"?' "$solution_path"/.meta/config.json; then
    release="--release"
fi

timeout -v 15s cargo test \
    --offline \
    $release \
    -- \
    -Z unstable-options \
    --include-ignored \
    --format json \
    2> >(
        # don't interpret the backticks in the sed command as expressions
        # shellcheck disable=SC2016
        #
        # This sed command fixes a flakiness issue for tests that fail to compile.
        # For the test example-syntax-error,
        # the following two output lines may appear in random order:
        #
        # error: could not compile `leap` (lib) due to 2 previous errors
        # error: could not compile `leap` (lib test) due to 2 previous errors
        #
        # Therefore, we remove the stuff in the parentheses.
        sed 's/could not compile `\(.*\)` (.*)/could not compile `\1`/g' >> "$output_path"/results.out
    ) | \
        /opt/test-runner/bin/transform-output \
        > "$output_path"/results.json

# Note: there is no test report output after the test timeout, so the transformer generates a generic "probable build failure" message.
# If we want a custom message for timeouts, we need to handle this case separately.
if grep -q "timeout: sending signal TERM" "$output_path"/results.out; then
   jq -n --rawfile m "$output_path"/results.out '{"version": 2, status: "error", message:"One of the tests timed out"}' > "$output_path"/results.json
elif grep -q "probable build failure" "$output_path"/results.json; then
   jq -n --rawfile m "$output_path"/results.out '{status: "error", message:$m}' > "$output_path"/results.json
fi