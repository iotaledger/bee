// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rand::{
    distributions::{
        uniform::{SampleRange, SampleUniform},
        Distribution, Standard,
    },
    Rng,
};

/// Generates a random integer.
pub fn rand_integer<T>() -> T
where
    Standard: Distribution<T>,
{
    rand::thread_rng().gen()
}

/// Generates a random integer within a given range.
pub fn rand_integer_range<T, R>(range: R) -> T
where
    T: SampleUniform + PartialOrd,
    R: SampleRange<T>,
{
    rand::thread_rng().gen_range(range)
}
