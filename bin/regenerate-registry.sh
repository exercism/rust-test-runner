#!/usr/bin/env bash

export RUSTUP_HOME=/usr/local/rustup
export CARGO_HOME=/usr/local/cargo
export PATH=/usr/local/cargo/bin:$PATH

cd /opt/test-runner/local-registry
perl -i -p0e 's/(\[dependencies\]\n).*/$1/se' Cargo.toml
curl -s -N "https://crates.io/api/v1/crates?page=1&per_page=100&sort=downloads" | jq -r '.crates[] | .id + "=\"" + .max_stable_version + "\""' >> Cargo.toml
cargo generate-lockfile && cargo local-registry --sync Cargo.lock .
