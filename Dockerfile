# always build this using the latest stable release
FROM rust:1.93.0 AS build-base

ARG JQ_VERSION=1.6
ARG JQ_URL=https://github.com/stedolan/jq/releases/download/jq-${JQ_VERSION}/jq-linux64
# download jq
RUN curl -L -o /usr/local/bin/jq "${JQ_URL}" \
    && chmod +x /usr/local/bin/jq

# install cargo-local-registry dependencies
RUN apt-get update && apt-get install -y gcc openssl cmake


FROM build-base AS build-rust-test-runner

RUN mkdir -p /rust-test-runner/src
ENV wd=/rust-test-runner
WORKDIR ${wd}
COPY Cargo.* ./
# for caching, we want to download and build all the dependencies before copying
# any of the real source files. We therefore build an empty dummy library,
# then remove it.
RUN echo '// dummy file' > src/lib.rs
RUN cargo build
# now get rid of the stub and copy the real source files
RUN rm src/lib.rs
COPY src/* src/
# build the executable
RUN cargo build --release


FROM build-base AS build-cargo-local-registry

# install cargo-local-registry
RUN cargo install --locked cargo-local-registry
# download popular crates to local registry
WORKDIR /local-registry
COPY local-registry/* ./
RUN cargo generate-lockfile && cargo local-registry --sync Cargo.lock .


# As of Dec 2019, we need to use the nightly toolchain to get JSON test output
# tracking issue: https://github.com/rust-lang/rust/issues/49359

# Official docker images with pinned nightly versions are not provided, but we
# want to pin the nightly version to avoid unnecessary cache misses. To achieve
# this, we copy the source of the official rust docker images and replace the
# version tag with a nightly one, pinned to a specific date.

# official Dockerfile source: 
# https://github.com/rust-lang/docker-rust/blob/master/stable/trixie/slim/Dockerfile

################ start-copy-pasta ################

FROM debian:trixie-slim

LABEL org.opencontainers.image.source=https://github.com/rust-lang/docker-rust

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=nightly-2026-01-22
#                ~~~~~~~~^~~~~~~~~~
#                 pin version here

RUN set -eux; \
    \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        gcc \
        libc6-dev \
        wget \
        ; \
    \
    arch="$(dpkg --print-architecture)"; \
    case "$arch" in \
        'amd64') \
            rustArch='x86_64-unknown-linux-gnu'; \
            rustupSha256='20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c'; \
            ;; \
        'armhf') \
            rustArch='armv7-unknown-linux-gnueabihf'; \
            rustupSha256='3b8daab6cc3135f2cd4b12919559e6adaee73a2fbefb830fadf0405c20231d61'; \
            ;; \
        'arm64') \
            rustArch='aarch64-unknown-linux-gnu'; \
            rustupSha256='e3853c5a252fca15252d07cb23a1bdd9377a8c6f3efa01531109281ae47f841c'; \
            ;; \
        'i386') \
            rustArch='i686-unknown-linux-gnu'; \
            rustupSha256='a5db2c4b29d23e9b318b955dd0337d6b52e93933608469085c924e0d05b1df1f'; \
            ;; \
        'ppc64el') \
            rustArch='powerpc64le-unknown-linux-gnu'; \
            rustupSha256='acd89c42b47c93bd4266163a7b05d3f26287d5148413c0d47b2e8a7aa67c9dc0'; \
            ;; \
        's390x') \
            rustArch='s390x-unknown-linux-gnu'; \
            rustupSha256='726b7fd5d8805e73eab4a024a2889f8859d5a44e36041abac0a2436a52d42572'; \
            ;; \
        'riscv64') \
            rustArch='riscv64gc-unknown-linux-gnu'; \
            rustupSha256='09e64cc1b7a3e99adaa15dd2d46a3aad9d44d71041e2a96100d165c98a8fd7a7'; \
            ;; \
        *) \
            echo >&2 "unsupported architecture: $arch"; \
            exit 1; \
            ;; \
    esac; \
    \
    url="https://static.rust-lang.org/rustup/archive/1.28.2/${rustArch}/rustup-init"; \
    wget --progress=dot:giga "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    \
    apt-get remove -y --auto-remove \
        wget \
        ; \
    rm -rf /var/lib/apt/lists/*; \
    \
    rustup --version; \
    cargo --version; \
    rustc --version;

################ end-copy-pasta ################

ENV wd=/opt/test-runner
RUN mkdir -p ${wd}/bin
WORKDIR ${wd}
COPY --from=build-rust-test-runner /rust-test-runner/target/release/rust_test_runner bin
COPY --from=build-base /usr/local/bin/jq /usr/local/bin
COPY --from=build-cargo-local-registry /local-registry local-registry/
# configure local-registry
RUN echo '[source.crates-io]\n\
    registry = "https://github.com/rust-lang/crates.io-index"\n\
    replace-with = "local-registry"\n\
    \n\
    [source.local-registry]\n\
    local-registry = "/opt/test-runner/local-registry/"\n' >> $CARGO_HOME/config.toml
# set entrypoint
COPY bin/run.sh bin
ENTRYPOINT ["bin/run.sh"]
