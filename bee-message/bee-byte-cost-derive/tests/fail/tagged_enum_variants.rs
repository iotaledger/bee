// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports, unreachable_patterns)]

use bee_byte_cost::ByteCost;

#[derive(ByteCost)]
enum SingleEnum {
    #[byte_cost(key)]
    SingleKey(u32),
}

fn main() {}
