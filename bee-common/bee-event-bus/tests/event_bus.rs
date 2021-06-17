// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_event_bus::EventBus;

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

struct Event(u64);
struct Bound0;
struct Bound1;
struct Bound2;

#[test]
fn add_listener_dispatch() {
    let bus = EventBus::new();
    let _counter = Arc::new(AtomicU64::new(0));

    let counter = _counter.clone();
    bus.add_listener::<(), _, _>(move |event: &Event| {
        counter.fetch_add(event.0, Ordering::SeqCst);
    });

    let counter = _counter.clone();
    bus.add_listener::<(), _, _>(move |event: &Event| {
        counter.fetch_add(2 * event.0, Ordering::SeqCst);
    });

    let counter = _counter.clone();
    bus.add_listener::<(), _, _>(move |event: &Event| {
        counter.fetch_add(3 * event.0, Ordering::SeqCst);
    });

    bus.dispatch(Event(1));
    bus.dispatch(Event(2));
    bus.dispatch(Event(3));

    assert_eq!(_counter.load(Ordering::SeqCst), 36);
}

#[test]
fn add_static_listener_dispatch() {
    let bus = EventBus::new();
    let _counter = Arc::new(AtomicU64::new(0));

    let counter = _counter.clone();
    bus.add_listener::<(), _, _>(move |event: &Event| {
        counter.fetch_add(event.0, Ordering::SeqCst);
    });

    bus.dispatch(Event(1));

    assert_eq!(_counter.load(Ordering::SeqCst), 1);
}

#[test]
fn add_listener_remove_dispatch() {
    let bus = EventBus::new();
    let _counter = Arc::new(AtomicU64::new(0));

    let counter = _counter.clone();
    bus.add_listener::<Bound0, _, _>(move |event: &Event| {
        counter.fetch_add(event.0, Ordering::SeqCst);
    });

    let counter = _counter.clone();
    bus.add_listener::<Bound1, _, _>(move |event: &Event| {
        counter.fetch_add(2 * event.0, Ordering::SeqCst);
    });

    let counter = _counter.clone();
    bus.add_listener::<Bound2, _, _>(move |event: &Event| {
        counter.fetch_add(3 * event.0, Ordering::SeqCst);
    });

    bus.remove_listeners::<Bound1>();

    bus.dispatch(Event(1));
    bus.dispatch(Event(2));
    bus.dispatch(Event(3));

    assert_eq!(_counter.load(Ordering::SeqCst), 24);
}
