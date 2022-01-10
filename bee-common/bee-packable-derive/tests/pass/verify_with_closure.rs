// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::{
    error::UnpackErrorExt,
    error::{UnknownTagError, UnpackError},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use core::convert::Infallible;

#[derive(Debug)]
pub struct PickyError(u8);

impl From<Infallible> for PickyError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

#[derive(Packable)]
#[packable(unpack_error = PickyError)]
pub struct Picky(
    #[packable(verify_with = |&value| { if !VERIFY && value == 42 { Ok(()) } else { Err(PickyError(value)) } })]
    u8
);

fn main() {}

