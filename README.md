# Rust Test Runner

Run unit tests for the Rust track of Exercism.

See:

- [`docker.md`](https://exercism.org/docs/building/tooling/test-runners/docker)
- [`interface.md`](https://exercism.org/docs/building/tooling/test-runners/interface)

## General Architecture

- The test runner itself is a simple shell script: `run.sh`
- The rust project in this repo is a filter which transforms Cargo's native testing output to the format expected by Exercism
- The dockerfile contains a multi-stage build: the first stage builds the test script, and the second stage uses it to run the student's solution and test suite and transform the output appropriately

## Run the test runner on an exercise using Docker

_This script is provided for testing purposes, as it mimics how test runners run in Exercism's production environment._

To run the tests of an arbitrary exercise using the Docker image, do the following:

1. Open a terminal in the project's root
2. Run `./bin/run-in-docker.sh <exercise-slug> <solution-dir> <output-dir>`

Once the test runner has finished, its results will be written to `<output-dir>/results.json`.

## Run the tests

To run the tests to verify the behavior of the test runner, do the following:

1. Open a terminal in the project's root
2. Run `./bin/run-tests.sh`

These are [golden tests][golden] that compare the `results.json` generated by running the current state of the code against the "known good" `tests/<test-name>/results.json`. All files created during the test run itself are discarded.

When you've made modifications to the code that will result in a new "golden" state, you'll need to generate and commit a new `tests/<test-name>/results.json` file.

## Run the tests using Docker

_This script is provided for testing purposes, as it mimics how test runners run in Exercism's production environment._

To run the tests to verify the behavior of the test runner using the Docker image, do the following:

1. Open a terminal in the project's root
2. Run `./bin/run-tests-in-docker.sh`

These are [golden tests][golden] that compare the `results.json` generated by running the current state of the code against the "known good" `tests/<test-name>/results.json`. All files created during the test run itself are discarded.

When you've made modifications to the code that will result in a new "golden" state, you'll need to generate and commit a new `tests/<test-name>/results.json` file.

[test-runners]: https://github.com/exercism/docs/tree/main/building/tooling/test-runners
[golden]: https://ro-che.info/articles/2017-12-04-golden-tests
[exercism]: https://exercism.io
