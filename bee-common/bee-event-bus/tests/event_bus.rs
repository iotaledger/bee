// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_event_bus::EventBus;

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

struct Event1(u64);
struct Event2(u64, u64);
struct Event3(u64, u64, u64);
struct Bound1;
struct Bound2;
struct Bound3;

#[test]
fn add_listener_dispatch() {
    let bus = EventBus::new();
    let _counter = Arc::new(AtomicU64::new(0));

    let counter = _counter.clone();
    bus.add_listener::<(), _, _>(move |event: &Event1| {
        counter.fetch_add(event.0, Ordering::SeqCst);
    });

    let counter = _counter.clone();
    bus.add_listener::<(), _, _>(move |event: &Event2| {
        counter.fetch_add(event.0 + event.1, Ordering::SeqCst);
    });

    let counter = _counter.clone();
    bus.add_listener::<(), _, _>(move |event: &Event3| {
        counter.fetch_add(event.0 + event.1 + event.2, Ordering::SeqCst);
    });

    bus.dispatch(Event1(1));
    bus.dispatch(Event1(2));
    bus.dispatch(Event2(1, 2));
    bus.dispatch(Event2(3, 4));
    bus.dispatch(Event3(1, 2, 3));
    bus.dispatch(Event3(4, 5, 6));

    assert_eq!(_counter.load(Ordering::SeqCst), 34);
}

#[test]
fn add_static_listener_dispatch() {
    let bus = EventBus::new();
    let _counter = Arc::new(AtomicU64::new(0));

    let counter = _counter.clone();
    bus.add_static_listener(move |event: &Event1| {
        counter.fetch_add(event.0, Ordering::SeqCst);
    });

    bus.dispatch(Event1(1));

    assert_eq!(_counter.load(Ordering::SeqCst), 1);
}

#[test]
fn add_listener_remove_dispatch() {
    let bus = EventBus::new();
    let _counter = Arc::new(AtomicU64::new(0));

    let counter = _counter.clone();
    bus.add_listener::<Bound1, _, _>(move |event: &Event1| {
        counter.fetch_add(event.0, Ordering::SeqCst);
    });

    let counter = _counter.clone();
    bus.add_listener::<Bound2, _, _>(move |event: &Event1| {
        counter.fetch_add(2 * event.0, Ordering::SeqCst);
    });

    let counter = _counter.clone();
    bus.add_listener::<Bound3, _, _>(move |event: &Event1| {
        counter.fetch_add(3 * event.0, Ordering::SeqCst);
    });

    bus.remove_listeners::<Bound2>();

    bus.dispatch(Event1(1));
    bus.dispatch(Event1(2));
    bus.dispatch(Event1(3));

    assert_eq!(_counter.load(Ordering::SeqCst), 24);
}
