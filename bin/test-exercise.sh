#!/usr/bin/env bash

set -e

exercise_path="$(realpath "$1")"
if [ -z "$exercise_path" ]; then
    echo "usage: $0 <path-to-exercise>"
    exit 1
fi

# bust the cache
touch "$exercise_path/src/lib.rs"

exercise_name="$(basename "$exercise_path")"

internal_mountpoint="/mnt/exercism-iteration"

cd "$(git rev-parse --show-toplevel)"
docker run \
    --rm \
    --volume "$exercise_path:$internal_mountpoint" \
    rtr "$exercise_name" "$internal_mountpoint"

mv "$exercise_path"/results.* .
jq . results.json
