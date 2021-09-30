#!/usr/bin/env bash

export RUSTUP_HOME=/usr/local/rustup
export CARGO_HOME=/usr/local/cargo
export PATH=/usr/local/cargo/bin:$PATH

cd /local-registry || cd /opt/test-runner/local-registry

perl -i -p0e 's/(\[dependencies\]\n).*/$1/se' Cargo.toml

# Uncomment after this file is added to exercism/rust (and remove file in current repo)
#curl -O -J https://raw.githubusercontent.com/exercism/rust/main/supported_crates

# this can't be parallelized, there's a lock on package cache when using cargo search
append_dep() {
  echo "retrieving info for $1..."
  cargo search "$1" | head -n 1 >> Cargo.toml
}
while IFS= read -r crate
    do append_dep "$crate"
done < supported_crates

cargo generate-lockfile && cargo local-registry --sync Cargo.lock .
