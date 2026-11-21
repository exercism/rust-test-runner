#!/usr/bin/env bash

# Synopsis:
# Compile the crates used by the track's exercises once, at image build, so
# that test runs don't compile them again.
#
# Each file in prebuilt-deps/ holds the dependency tables of one exercise's
# Cargo.toml. For each file, this script makes a small package with those
# dependencies and runs `cargo test --no-run` on it, all into one shared
# target directory. Cargo gives every compiled crate a hash of its version,
# features and compiler flags, so the same crate built for different
# exercises is stored once, and a crate with other features is stored
# separately.
#
# At the start of each run, the test runner links the files from this
# directory into the solution's target directory (see src/dep_cache.rs).
# Cargo then sees those crates as already built and only compiles the
# solution and its tests.

# Example:
# ./bin/build-dep-cache.sh prebuilt-deps /opt/test-runner/dep-cache

set -euo pipefail

manifests=$(realpath "$1")
export CARGO_TARGET_DIR=$2

# The package name must not clash with any crate in the cache, because its
# own files are removed at the end.
package=exercism_prebuilt_deps

work=$(mktemp -d)
cd "$work"
mkdir src
touch src/lib.rs

for manifest in "$manifests"/*.toml; do
    {
        printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2024"\n\n' "$package"
        cat "$manifest"
    } > Cargo.toml
    rm -f Cargo.lock
    cargo test --offline --no-run
done

# Keep only the dependencies. The package's own files, its incremental
# compilation data and cargo's lock file are of no use to a run.
cd "$CARGO_TARGET_DIR/debug"
find . -maxdepth 2 -name "*${package}*" -exec rm -rf {} +
rm -rf incremental .cargo-lock "$CARGO_TARGET_DIR/.rustc_info.json" \
    "$CARGO_TARGET_DIR/CACHEDIR.TAG"
rm -rf "$work"
