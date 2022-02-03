// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_byte_cost::{min_deposit, ByteCost, ByteCostConfig};
use packable::Packable;

use core::convert::Infallible;

const CONFIG: ByteCostConfig = ByteCostConfig {
    byte_cost: 1,
    weight_for_data: 1,
    weight_for_key: 10,
};

#[derive(Default, ByteCost, Packable)]
#[packable(unpack_error = Infallible)]
struct BothKeysData {
    #[byte_cost(key)]
    a: u32,
    #[byte_cost(data)]
    b: u32,
}

#[derive(Default, ByteCost, Packable)]
struct OnlyData {
    #[byte_cost(data)]
    a: u32,
    #[byte_cost(data)]
    b: u32,
}

#[derive(Default, ByteCost, Packable)]
struct OnlyKeys {
    #[byte_cost(key)]
    a: u32,
    #[byte_cost(key)]
    b: u32,
}

#[derive(Default, ByteCost, Packable)]
struct Mixed {
    #[byte_cost(key, data)]
    a: u32,
}

#[test]
fn flat_fields() {
    assert_eq!(min_deposit(&CONFIG, &OnlyData::default()), 8);
    assert_eq!(min_deposit(&CONFIG, &OnlyKeys::default()), 80);
    assert_eq!(min_deposit(&CONFIG, &BothKeysData::default()), 44);
    assert_eq!(min_deposit(&CONFIG, &Mixed::default()), 44);
}

#[test]
fn unnamed_fields() {
    #[derive(Default, ByteCost, Packable)]
    struct Unnamed(#[byte_cost(key)] u32, #[byte_cost(data)] u32);
    assert_eq!(min_deposit(&CONFIG, &Unnamed::default()), 44);
}

#[test]
fn nested_fields() {
    #[derive(Default, ByteCost, Packable)]
    struct Parent {
        #[byte_cost(data)]
        a: u32,
        b: BothKeysData,
    }
    assert_eq!(min_deposit(&CONFIG, &Parent::default()), 48);
}

#[test]
fn arrays() {
    #[derive(Default, ByteCost)]
    struct Parent {
        b: [BothKeysData; 3],
    }
    assert_eq!(min_deposit(&CONFIG, &Parent::default()), 44 * 3);
}

#[test]
fn enums() {
    #[repr(u8)]
    #[derive(ByteCost)]
    enum Parent {
        Keys(OnlyKeys),
        Data(OnlyData),
    }
    assert_eq!(min_deposit(&CONFIG, &Parent::Data(Default::default())), 8 + 1);
    assert_eq!(min_deposit(&CONFIG, &Parent::Keys(Default::default())), 80 + 1);
}
