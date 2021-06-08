# `bee-ord`

This crate contains functionality used for checking the ordering of `Iterator`s. This is required by the node in places such as message validation.

Rust provides some ordering methods with the `Iterator` trait, but there are a number of them (including [`is_sorted`](https://doc.rust-lang.org/core/iter/trait.Iterator.html#method.is_sorted)) that are currently nightly-only, so we cannot use them here (since we want the node to use the stable toolchain). Instead, we have implemented our own simple functions in this crate.