#!/bin/sh

# Print and run a command
print_and_run() {
    cmd=\`$@\`
    echo Running $cmd...
    $*
    status_code=$?
    if [[ $status_code -eq 0 ]] ; then
        echo $cmd was succesful!
    else
        echo $cmd failed!
    fi
    return $status_code
}

#
if [[ $# -eq 0 ]] ; then
    packages=--all
else
    packages=-p $*
fi

print_and_run cargo +nightly fmt $packages &&
print_and_run cargo check --all-features $packages &&
print_and_run cargo +nightly check --all-features $packages &&
print_and_run cargo +nightly clippy --all-targets --all-features $packages -- -D warnings &&
print_and_run cargo test --all-features --no-fail-fast $packages &&
print_and_run cargo +nightly test --all-features --no-fail-fast $packages &&
print_and_run cargo doc --all-features $packages &&
print_and_run cargo +nightly udeps --all-features $packages
