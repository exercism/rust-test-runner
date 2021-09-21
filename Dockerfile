# always build this using the latest stable release
FROM rust:latest as build

RUN mkdir -p /rust-test-runner/src
WORKDIR /rust-test-runner
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
RUN mkdir -p ${wd}/bin
RUN curl -L -o /usr/local/bin/jq https://github.com/stedolan/jq/releases/download/jq-1.6/jq-linux64 \
  && chmod +x /usr/local/bin/jq
# download and build popular crates to local registry
RUN mkdir /local-registry
WORKDIR /local-registry
COPY local-registry/* ./
RUN curl "https://crates.io/api/v1/crates?page=1&per_page=100&sort=downloads" | \
    jq -r '.crates[] | .id + "=\"" + .max_stable_version + "\""' >> Cargo.toml
RUN cargo generate-lockfile && \
    cargo install cargo-local-registry && \
    cargo local-registry --sync Cargo.lock .

# As of Dec 2019, we need to use the nightly toolchain to get JSON test output
FROM rustlang/rust:nightly AS test
ENV wd /opt/test-runner
RUN mkdir -p ${wd}/bin
WORKDIR ${wd}
COPY --from=build /rust-test-runner/target/release/transform-output bin
COPY --from=build /usr/local/bin/jq bin
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
