# always build this using the latest stable release
FROM rust:latest as build

RUN mkdir -p /rust-test-runner/src
WORKDIR /rust-test-runner
COPY Cargo.* .
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

# As of Aug 2019, we need to use the nightly build to get JSON test output
FROM rustlang/rust:nightly AS test
RUN mkdir -p /opt/test-runner/bin
COPY --from build /rust-test-runner/target/release/rust-test-runner /opt/test-runner/bin/transform-output
COPY run.sh /opt/test-runner/bin/
ENTRYPOINT ['/opt/test-runner/bin/run.sh']