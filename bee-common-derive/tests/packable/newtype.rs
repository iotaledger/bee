// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_common_derive::Packable;

#[derive(Packable)]
pub struct Num(u32);

fn main() {}
