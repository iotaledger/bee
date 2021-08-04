// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::{error::UnknownTagError, Packable};

use core::convert::Infallible;

#[derive(Packable)]
#[packable(tag_type = u8, with_error = InvalidTag::new)]
#[packable(pack_error = Infallible)]
#[packable(unpack_error = InvalidTag)]
pub enum OptI32 {
    #[packable(tag = 0)]
    None,
    #[packable(tag = 1)]
    Some(i32),
}

pub struct InvalidTag(u8);

impl InvalidTag {
    fn new(tag: u8) -> Self {
        Self(tag)
    }
}

impl From<Infallible> for InvalidTag {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

fn main() {}
