# always build this using the latest stable release
FROM rust:latest as build

ARG CLR_NAME=cargo-local-registry
ARG CLR_VERSION=0.2.2
ARG CLR_URL=https://github.com/dhovart/${CLR_NAME}/releases/download/${CLR_VERSION}/${CLR_NAME}-${CLR_VERSION}-x86_64-unknown-linux-musl.tar.gz

ARG JQ_VERSION=1.6
ARG JQ_URL=https://github.com/stedolan/jq/releases/download/jq-${JQ_VERSION}/jq-linux64

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
# retrieve cargo-local-registry
RUN curl -L -o clr.tar.gz "${CLR_URL}" \
 && tar xvzf clr.tar.gz --strip-components=1 \
 && chmod +x cargo-local-registry \
 && mv cargo-local-registry /usr/local/cargo/bin
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
