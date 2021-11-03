// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::Packable;

use core::convert::Infallible;

#[derive(Packable)]
#[packable(unpack_error = Impossible, with = Impossible::new)]
pub struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
pub enum Impossible {}

impl Impossible {
    fn new() -> Self {
        unreachable!()
    }
}

fn main() {}
