// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use bee_type_id::TypeId;

#[derive(TypeId)]
pub struct Foo {
    a: u8,
    b: String,
    c: [u8; 32],
}

#[derive(TypeId)]
pub struct Bar<T> {
    inner: Vec<T>,
}

#[test]
fn unique_id() {
    assert!(Foo::type_id() != Bar::<u64>::type_id());
}
