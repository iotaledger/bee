// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports, unreachable_patterns)]

use bee_byte_cost::ByteCost;

#[derive(Default, ByteCost)]
struct SingleField {
    a: u32,
}

fn main() {}
