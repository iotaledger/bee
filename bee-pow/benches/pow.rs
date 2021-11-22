// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use bee_common::packable::Packable;
use bee_pow::{
    providers::{miner::MinerBuilder, NonceProvider, NonceProviderBuilder},
    score::PoWScorer,
};
use bee_test::rand::message::rand_message;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

fn scoring_benchmark(c: &mut Criterion) {
    let mut pow = PoWScorer::new();
    c.bench_function("scoring", |b| {
        b.iter_batched(
            rand_message,
            |msg| {
                let msg_bytes = msg.pack_new();
                pow.score(&msg_bytes)
            },
            BatchSize::SmallInput,
        );
    });
}

fn mining_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("mining");
    g.measurement_time(Duration::from_secs(60));

    g.bench_function("single-threaded", |b| {
        let miner = MinerBuilder::default().with_num_workers(1).finish();
        b.iter_batched(
            rand::random::<[u8; 32]>,
            |bytes| miner.nonce(&bytes, 4000.0),
            BatchSize::SmallInput,
        )
    });

    g.bench_function("multi-threaded", |b| {
        let miner = MinerBuilder::default().with_num_workers(num_cpus::get()).finish();
        b.iter_batched(
            rand::random::<[u8; 32]>,
            |bytes| miner.nonce(&bytes, 4000.0),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, scoring_benchmark, mining_benchmark);
criterion_main!(benches);
