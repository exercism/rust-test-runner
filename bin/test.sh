#!/usr/bin/env bash

set -e
rbase="$(git rev-parse --show-toplevel)"

expath="$1"
if [ -z "$expath" ]; then
    expath="$(pwd)"
    slug="rtr"
else
    cd "$expath"
    expath="$(pwd)"
    slug="$(basename "$expath")"
fi

cd "$rbase"
docker build . --tag rtr
docker run --rm \
    --volume "$expath":/mnt/exercism-iteration/ \
    --volume "$rbase":/mnt/output/ \
    rtr \
    "$slug" \
    /mnt/exercism-iteration/ \
    /mnt/output
