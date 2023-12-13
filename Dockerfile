# always build this using the latest stable release
FROM rust:1.74.1 as build


ARG JQ_VERSION=1.6
ARG JQ_URL=https://github.com/stedolan/jq/releases/download/jq-${JQ_VERSION}/jq-linux64

# install cargo-local-registry dependencies
RUN apt-get update && apt-get install -y gcc openssl cmake

RUN mkdir -p /rust-test-runner/src
ENV wd /rust-test-runner
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
# download jq
RUN curl -L -o /usr/local/bin/jq "${JQ_URL}" \
    && chmod +x /usr/local/bin/jq
# install cargo-local-registry
RUN cargo install cargo-local-registry
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
# https://github.com/rust-lang/docker-rust/blob/master/1.74.1/bookworm/slim/Dockerfile

################ start-copy-pasta ################

FROM debian:bookworm-slim

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=nightly-2023-12-12
#                ~~~~~~~~^~~~~~~~~~
#                 pin version here

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        gcc \
        libc6-dev \
        wget \
        ; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='0b2f6c8f85a3d02fde2efc0ced4657869d73fccfce59defb4e8d29233116e6db' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='f21c44b01678c645d8fbba1e55e4180a01ac5af2d38bcbd14aa665e0d96ed69a' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='673e336c81c65e6b16dcdede33f4cc9ed0f08bde1dbe7a935f113605292dc800' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='e7b0f47557c1afcd86939b118cbcf7fb95a5d1d917bdd355157b63ca00fc4333' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.26.0/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    apt-get remove -y --auto-remove \
        wget \
        ; \
    rm -rf /var/lib/apt/lists/*;

################ end-copy-pasta ################

ENV wd /opt/test-runner
RUN mkdir -p ${wd}/bin
WORKDIR ${wd}
COPY --from=build /rust-test-runner/target/release/rust_test_runner bin
COPY --from=build /usr/local/bin/jq /usr/local/bin
# configure local-registry
COPY --from=build /local-registry local-registry/
RUN echo '[source.crates-io]\n\
    registry = "https://github.com/rust-lang/crates.io-index"\n\
    replace-with = "local-registry"\n\
    \n\
    [source.local-registry]\n\
    local-registry = "/opt/test-runner/local-registry/"\n' >> $CARGO_HOME/config.toml
# set entrypoint
COPY bin/run.sh bin
ENTRYPOINT ["bin/run.sh"]
