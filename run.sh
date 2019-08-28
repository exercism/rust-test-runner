#!/usr/bin/env bash

slug="$1"
solution_path="$2"

echo "slug:          $slug"
echo "solution path: $solution_path"

if [ -z "$slug" ] || [ -z "$solution_path" ]; then
    echo "slug and solution path must be present"
    exit 1
fi
