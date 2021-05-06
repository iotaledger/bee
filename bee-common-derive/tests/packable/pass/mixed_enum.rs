// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_common::packable::Packable;
use bee_common_derive::Packable;

#[derive(Packable)]
#[packable(ty = u8)]
pub enum Foo {
    #[packable(id = 0)]
    Bar(u32),
    #[packable(id = 1)]
    Baz{ x: i32, y: i32 }
}

fn main() {}
