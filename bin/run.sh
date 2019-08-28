#!/usr/bin/env bash

set -e

slug="$1"
solution_path="$2"

echo "slug:          $slug"
echo "solution path: $solution_path"

if [ -z "$slug" ] || [ -z "$solution_path" ]; then
    echo "slug and solution path must be present"
    exit 1
fi

cd "$solution_path"
cargo +nightly test -- -Z unstable-options --format json |\
    /opt/test-runner/bin/transform-output
