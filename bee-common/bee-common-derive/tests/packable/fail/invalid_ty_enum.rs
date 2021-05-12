// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_common::packable::Packable;

use core::convert::Infallible;

#[derive(Packable)]
#[packable(tag_ty = [u8; 32])]
#[packable(error = Infallible)]
pub enum OptI32 {
    #[packable(tag = 0)]
    None,
    #[packable(tag = 1)]
    Some(i32)
}

fn main() {}
