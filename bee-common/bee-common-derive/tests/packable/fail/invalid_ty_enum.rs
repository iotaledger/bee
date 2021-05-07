// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_common::packable::Packable;
use bee_common_derive::Packable;

#[derive(Packable)]
#[packable(ty = [u8; 32])]
pub enum OptI32 {
    #[packable(id = 0)]
    None,
    #[packable(id = 1)]
    Some(i32)
}

fn main() {}
