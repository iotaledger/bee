// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]
use bee_common::packable::Packable;
use bee_common_derive::Packable;

#[derive(Packable)]
pub struct Point {
    x: i32,
    y: i32,
}

fn main() {}
