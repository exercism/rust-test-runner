#!/usr/bin/env bash

set -e

cd "$(git rev-parse --show-toplevel)"
docker build . --tag rtr
docker run --rm --volume "$(pwd)":/mnt/exercism-iteration/ rtr rtr /mnt/exercism-iteration/
