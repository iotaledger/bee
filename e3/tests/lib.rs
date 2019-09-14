//! IMPORTANT NOTE
//!
//! The integration tests share state because they are all using the same messaging sytem which is initialized statically.
//! That's why the tests will fail when executed in parallel because they get in eachother's way.
//!
//! Use `cargo test --release -- --test-threads=1` to ensure all tests are executed one after the other.
mod basic;
