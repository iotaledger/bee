// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports, unreachable_patterns)]

use bee_byte_cost::ByteCost;

#[derive(Default, ByteCost)]
struct SingleField {
    #[byte_cost(key)]
    #[byte_cost(data)]
    a: u32,
}

fn main() {}
