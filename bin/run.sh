#!/usr/bin/env bash

set -e

export RUSTUP_HOME=/usr/local/rustup
export CARGO_HOME=/usr/local/cargo
export PATH=/usr/local/cargo/bin:$PATH

# Call cargo, rustc and rustdoc from the toolchain directly. Going through the
# rustup proxies in /usr/local/cargo/bin starts an extra process for every
# call, and reads about 10 MB more from disk when the cache is cold.
toolchain=/usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu
if [ -x "$toolchain/bin/cargo" ]; then
    export PATH="$toolchain/bin:$PATH"
fi

/opt/test-runner/bin/rust_test_runner "$@"
