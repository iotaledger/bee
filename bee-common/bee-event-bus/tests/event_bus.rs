// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_event_bus::EventBus;

use std::any::TypeId;

#[test]
fn test() {
    let _bus = EventBus::<TypeId>::new();
}
