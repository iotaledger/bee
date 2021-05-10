// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_common::packable::Packable;
use bee_common_derive::Packable;

use core::convert::Infallible;

#[derive(Packable)]
#[packable(tag_ty = u8)]
#[packable(error = Infallible)]
pub enum Foo {
    #[packable(tag = 0)]
    Bar(u32),
    #[packable(tag = 1)]
    Baz{ x: i32, y: i32 }
}

fn main() {}
