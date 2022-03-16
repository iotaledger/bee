#!/bin/bash

# This script can be used to run `cargo` commands on each of the crates of the workspace individually.
# It expects two inputs:
#   $1: The cargo command
#   $2: Any additional flags as string

# Example: `.github/workflows/scripts/for_each_crate.sh build "--all-targets --all-features --release"`

set -e # Abort script on first error.

CRATES=`cargo metadata --format-version 1 --no-deps | jq .packages[].name`

for PACKAGE in ${CRATES}
do
    UNQUOTED=`echo "$PACKAGE" | tr -d '"'`
    echo "cargo $1 --package ${UNQUOTED} $2"
    cargo $1 -p ${UNQUOTED} $2;
done
