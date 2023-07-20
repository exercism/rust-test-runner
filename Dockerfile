# always build this using the latest stable release
FROM rust:1.71.0 as build


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
COPY bin/generate-registry.sh ${wd}/bin/
# download jq
RUN curl -L -o /usr/local/bin/jq "${JQ_URL}" \
    && chmod +x /usr/local/bin/jq
# install cargo-local-registry
RUN cargo install cargo-local-registry
# download popular crates to local registry
WORKDIR /local-registry
COPY local-registry/* ./
RUN ${wd}/bin/generate-registry.sh

# As of Dec 2019, we need to use the nightly toolchain to get JSON test output
# tracking issue: https://github.com/rust-lang/rust/issues/49359

# Official docker images with pinned nightly versions are not provided, but we
# want to pin the nightly version to avoid unnecessary cache misses. To achieve
# this, we copy the source of the official rust docker images and replace the
# version tag with a nightly one, pinned to a specific date.

# official Dockerfile source: 
# https://github.com/rust-lang/docker-rust/blob/master/1.71.0/alpine3.18/Dockerfile

################ start-copy-pasta ################

FROM alpine:3.18

RUN apk add --no-cache \
        ca-certificates \
        gcc

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=nightly-2023-07-18
#                ~~~~~~~~^~~~~~~~~~
#                 pin version here

RUN set -eux; \
    apkArch="$(apk --print-arch)"; \
    case "$apkArch" in \
        x86_64) rustArch='x86_64-unknown-linux-musl'; rustupSha256='7aa9e2a380a9958fc1fc426a3323209b2c86181c6816640979580f62ff7d48d4' ;; \
        aarch64) rustArch='aarch64-unknown-linux-musl'; rustupSha256='b1962dfc18e1fd47d01341e6897cace67cddfabf547ef394e8883939bd6e002e' ;; \
        *) echo >&2 "unsupported architecture: $apkArch"; exit 1 ;; \
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
    rustc --version;

################ end-copy-pasta ################

ENV wd /opt/test-runner
RUN mkdir -p ${wd}/bin
WORKDIR ${wd}
COPY --from=build /rust-test-runner/target/release/transform-output bin
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
COPY --from=build /usr/local/cargo/bin/cargo-local-registry /usr/local/cargo/bin/
COPY bin/generate-registry.sh bin
ENTRYPOINT ["bin/run.sh"]
