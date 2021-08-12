// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_event_bus::UniqueId;

use std::any::TypeId;

#[test]
fn from_type_id() {
    let unique_id = UniqueId::<u8>::from(TypeId::of::<u8>());

    assert_eq!(unique_id, UniqueId::Type(TypeId::of::<u8>()));
}

#[test]
fn derived_impls() {
    let unique_id = UniqueId::Object(42u8);
    let unique_id_copy = unique_id;
    let unique_id_clone = unique_id.clone();
    let unique_id_debug = format!("{:?}", unique_id);

    assert_eq!(unique_id_copy, unique_id);
    assert_eq!(unique_id_clone, unique_id);
    assert_eq!(unique_id_debug, "Object(42)");
}
