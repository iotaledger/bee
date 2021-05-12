// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_common::packable::{UnknownTagError, Packable};
use bee_common_derive::Packable;

#[derive(Packable)]
#[packable(tag_ty = u8)]
#[packable(error = UnknownTagError<u8>)]
pub enum OptPoint {
    #[packable(tag = 0)]
    None,
    #[packable(tag = 1)]
    Some{ x: i32, y: i32 }
}

fn main() {}
