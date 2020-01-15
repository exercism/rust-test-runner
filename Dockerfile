FROM rust:1.40.0 as build

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

# As of Dec 2019, we need to use the nightly toolchain to get JSON test output
# FROM rustlang/rust:nightly AS test
FROM rustlang/rust:b4f4f4589b1108f38f83a91e009761a182bdb4e4f12f03e508352f5fb9154910
ENV wd /opt/test-runner
RUN mkdir -p ${wd}/bin
WORKDIR ${wd}
COPY --from=build /rust-test-runner/target/release/transform-output bin
COPY bin/run.sh bin
ENTRYPOINT ["bin/run.sh"]
