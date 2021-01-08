// Copyright 2020 IOTA Stiftung
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

    assert_eq!(received.load(Ordering::SeqCst), true);
}

#[test]
fn send_sync() {
    fn helper<T: Send + Sync>() {}
    helper::<Bus<'static>>();
}

// TODO: Enable when stable
// #[bench]
// fn bench_add_two(b: &mut Bencher) {
// use std::hint::black_box;
//
// let bus = Bus::default();
//
// bus.add_listener(|e: &Foo| { black_box(e); });
//
// b.iter(|| {
// bus.dispatch(Foo);
// });
// }
