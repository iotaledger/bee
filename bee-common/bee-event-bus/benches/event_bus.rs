// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_event_bus::EventBus;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

struct Event;

fn listener(_: &Event) {}

fn dispatch(bus: &EventBus) {
    bus.dispatch(Event {});
}

fn event_bus(c: &mut Criterion) {
    let bus = EventBus::new();
    let mut group = c.benchmark_group("EventBus");

    bus.add_static_listener(listener);

    group.throughput(Throughput::Elements(1000));
    group.bench_with_input(BenchmarkId::new("Dispatch", "Event"), &bus, |bencher, bus| {
        bencher.iter(|| dispatch(bus))
    });

    group.finish();
}

criterion_group!(benches, event_bus);
criterion_main!(benches);
