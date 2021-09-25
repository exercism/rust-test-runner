# always build this using the latest stable release
FROM rust:latest as build

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
RUN mkdir -p ${wd}/bin
COPY bin/generate-registry.sh ${wd}/bin
# download jq
RUN curl -L -o /usr/local/bin/jq https://github.com/stedolan/jq/releases/download/jq-1.6/jq-linux64 \
  && chmod +x /usr/local/bin/jq
# download cargo-local-registry
RUN curl -L -o clr.tar.gz https://github.com/ChrisGreenaway/cargo-local-registry/releases/download/0.2.1/cargo-local-registry-0.2.1-x86_64-unknown-linux-musl.tar.gz \
  && tar xvzf clr.tar.gz && chmod +x cargo-local-registry && mv cargo-local-registry /usr/local/cargo/bin
# download popular crates to local registry
WORKDIR /local-registry
COPY local-registry/* ./
RUN ${wd}/bin/generate-registry.sh

# As of Dec 2019, we need to use the nightly toolchain to get JSON test output
FROM rustlang/rust:nightly AS test
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
