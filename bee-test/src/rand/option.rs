// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::bool::rand_bool;

/// Generates a random [`Option`] from a given generator.
pub fn rand_option<T, F>(f: F) -> Option<T>
where
    F: FnOnce() -> T,
{
    if rand_bool() { Some(f()) } else { None }
}
