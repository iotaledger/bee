// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rand::{
    distributions::{uniform::SampleUniform, Distribution, Standard},
    Rng,
};

use std::ops::Range;

pub fn rand_integer<T>() -> T
where
    Standard: Distribution<T>,
{
    rand::thread_rng().gen()
}

pub fn rand_integer_range<T>(range: Range<T>) -> T
where
    T: SampleUniform + PartialOrd,
{
    rand::thread_rng().gen_range(range)
}
