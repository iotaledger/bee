#! /bin/bash

# Ensure past coverage files are deleted (useful for running locally)
echo "Removing stale coverage files..."
find . -name "*.profraw" -type f -delete
find . -name "*.profdata" -type f -delete
rm -r coverage
mkdir coverage

# Switch to nightly toolchain, in case running locally
rustup override set nightly

# Run tests with profiling instrumentation
echo "Running instrumented unit tests..."
RUSTFLAGS="-Zinstrument-coverage" LLVM_PROFILE_FILE="bee-%m.profraw" cargo test --tests --all --all-features

# Merge all .profraw files into "bee.profdata"
echo "Merging coverage data..."
cargo profdata -- merge */bee-*.profraw -o bee.profdata

# List the test binaries
echo "Locating test binaries..."
BINARIES=""

for file in \
  $( \
    RUSTFLAGS="-Zinstrument-coverage" \
      cargo test --tests --all --all-features --no-run --message-format=json \
        | jq -r "select(.profile.test == true) | .filenames[]" \
        | grep -v dSYM - \
  ); \
do
  echo "Found $file"
  BINARIES="${BINARIES} -object $file"
done

# Generate and export the coverage report to lcov format
echo "Generating lcov file..."
cargo cov -- export ${BINARIES} \
  --instr-profile=bee.profdata \
  --ignore-filename-regex="/.cargo|rustc|target|tests|/.rustup" \
  --format=lcov --Xdemangler=rustfilt \
  >> coverage/coverage.info