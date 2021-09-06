// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Generates a random [`Vec`] from a given generator and length.
pub fn rand_vec<T, F>(f: F, length: usize) -> Vec<T>
where
    F: Fn() -> T,
{
    (0..length).map(|_| f()).collect()
}
