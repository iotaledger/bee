// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::number::rand_number_range;

use rand::distributions::uniform::SampleRange;

/// Generates a random `Vec` with a random length in a given range.
pub fn vec_rand_length<T, R, F>(len_range: R, f: F) -> Vec<T>
where
    T: Clone,
    R: SampleRange<usize>,
    F: FnOnce() -> T,
{
    vec![f(); rand_number_range(len_range)]
}
