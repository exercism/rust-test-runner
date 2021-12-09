#!/usr/bin/env bash

MANIFEST=local-registry/Cargo.toml
DEP_LINE=$(grep -n "\[dependencies\]" $MANIFEST | cut -d : -f 1)
DEPS=$(tail -n +$(($DEP_LINE + 1)) $MANIFEST)

while IFS= read -r line
do
    crate=$(echo "$line" | awk -F"[ ]+" '{print $1}')
    >&2 echo "Retrieving latest version of $crate..."
    latest_version=$(cargo search "$crate" | head -n 1 | awk -F"[ ]+" '{print $1  " = " $3}')
    sed -i -r "s/^$crate =.*/$latest_version/" $MANIFEST
 done <<< "$DEPS"
