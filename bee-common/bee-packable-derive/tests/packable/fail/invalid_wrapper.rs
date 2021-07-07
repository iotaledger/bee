// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::{VecPrefix, Packable};

use core::convert::Infallible;

#[derive(Packable)]
#[packable(pack_error = Infallible)]
#[packable(unpack_error = Infallible)]
pub struct Foo {
    #[packable(wrapper = VecPrefix<u64, u8, 128>)]
    inner: Vec<u32>,
}

fn main() {}

