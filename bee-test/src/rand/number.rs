// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rand::{
    distributions::{
        uniform::{SampleRange, SampleUniform},
        Distribution, Standard,
    },
    Rng,
};

/// Generates a random number.
pub fn rand_number<T>() -> T
where
    Standard: Distribution<T>,
{
    rand::thread_rng().gen()
}

/// Generates a random number within a given range.
pub fn rand_number_range<T, R>(range: R) -> T
where
    T: SampleUniform + PartialOrd,
    R: SampleRange<T>,
{
    rand::thread_rng().gen_range(range)
}
