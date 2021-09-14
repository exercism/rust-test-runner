# Rust Test Runner

Run unit tests for the Rust track of Exercism.

See:

- [`docker.md`](https://exercism.org/docs/building/tooling/test-runners/docker)
- [`interface.md`](https://exercism.org/docs/building/tooling/test-runners/interface)

## General Architecture

- The test runner itself is a simple shell script: `run.sh`
- The rust project in this repo is a filter which transforms Cargo's native testing output to the format expected by Exercism
- The dockerfile contains a multi-stage build: the first stage builds the test script, and the second stage uses it to run the student's solution and test suite and transform the output appropriately
