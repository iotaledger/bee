// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_runtime::event::Bus;

struct Foo;

#[test]
fn basic() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let bus = Bus::default();

    let received = AtomicBool::new(false);

    bus.add_static_listener::<_, _>(|_: &Foo| received.store(true, Ordering::SeqCst));

    bus.dispatch(Foo);

    drop(bus);

    assert!(received.load(Ordering::SeqCst));
}

#[test]
fn send_sync() {
    fn helper<T: Send + Sync>() {}
    helper::<Bus<'static>>();
}
