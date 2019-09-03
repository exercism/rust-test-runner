# Rust Test Runner

Run unit tests for the Rust track of Exercism.

See:

- [`rules.md`](https://github.com/exercism/automated-tests/blob/abaa1a75c2b259f12485c6dcdf186c66cc547afa/docs/rules.md)
- [`docker.md`](https://github.com/exercism/automated-tests/blob/master/docs/docker.md)
- [`interface.md`](https://github.com/exercism/automated-tests/blob/d4f221f63edd2392bcc12408f995e90c1a4c2c85/docs/interface.md)

## General Architecture

- The test runner itself is a simple shell script: `run.sh`
- The rust project in this repo is a filter which transforms Cargo's native testing output to the format expected by Exercism
- The dockerfile contains a multi-stage build: the first stage builds the test script, and the second stage uses it to run the student's solution and test suite and transform the output appropriately
