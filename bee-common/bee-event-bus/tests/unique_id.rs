// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_event_bus::UniqueId;

use std::any::TypeId;

#[test]
fn from_type_id() {
    let _unique_id = UniqueId::<u8>::from(TypeId::of::<u8>());
}

#[test]
fn derived_impls() {
    let _unique_id = UniqueId::Object(42u8);
    let _unique_id_copy = _unique_id;
    let _unique_id_clone = _unique_id.clone();
    let _unique_id_debug = format!("{:?}", _unique_id);

    assert_eq!(_unique_id_copy, _unique_id);
    assert_eq!(_unique_id_clone, _unique_id);
}
