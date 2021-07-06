// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::Packable;

use core::convert::Infallible;

#[derive(Packable)]
#[packable(tag_type = [u8; 32])]
#[packable(pack_error = Infallible)]
#[packable(unpack_error = Infallible)]
pub enum OptI32 {
    #[packable(tag = 0)]
    None,
    #[packable(tag = 1)]
    Some(i32),
}

fn main() {}
