// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rand::{
    distributions::{
        uniform::{SampleRange, SampleUniform},
        Distribution, Standard,
    },
    Rng,
};

pub fn rand_integer<T>() -> T
where
    Standard: Distribution<T>,
{
    rand::thread_rng().gen()
}

pub fn rand_integer_range<T, R>(range: R) -> T
where
    T: SampleUniform + PartialOrd,
    R: SampleRange<T>,
{
    rand::thread_rng().gen_range(range)
}
