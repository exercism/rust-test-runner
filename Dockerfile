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
RUN curl -L -o jq https://github.com/stedolan/jq/releases/download/jq-1.6/jq-linux64 \
  && chmod +x jq

# As of Dec 2019, we need to use the nightly toolchain to get JSON test output
FROM rustlang/rust:nightly AS test
ENV wd /opt/test-runner
RUN mkdir -p ${wd}/bin
WORKDIR ${wd}
COPY --from=build /rust-test-runner/target/release/transform-output /rust-test-runner/jq bin/
COPY bin/run.sh bin
ENTRYPOINT ["bin/run.sh"]
