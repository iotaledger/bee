// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::{error::UnknownTagError, Packable};

use core::convert::Infallible;

#[derive(Packable)]
#[packable(tag_type = u8)]
#[packable(unpack_error = UnknownTagError<u8>)]
pub enum OptI32 {
    #[packable(tag = 0u32)]
    None,
    #[packable(tag = 1)]
    Some(i32),
}

fn main() {}
