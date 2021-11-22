// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use bee_common::packable::Packable;
use bee_pow::{
    providers::{miner::MinerBuilder, NonceProvider, NonceProviderBuilder},
    score::PoWScorer,
};
use bee_test::rand::message::rand_message;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};

const MINIMUM_POW_SCORE: f64 = 4000.0;

fn scoring_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring");
    g.throughput(Throughput::Elements(1));

    g.bench_function("random message", |b| {
        let mut pow = PoWScorer::new();
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
    let mut g = c.benchmark_group("find nonce");
    g.measurement_time(Duration::from_secs(120));
    g.throughput(Throughput::Elements(1));

    g.bench_function("random message (single-threaded)", |b| {
        let miner = MinerBuilder::default().with_num_workers(1).finish();
        b.iter_batched(
            // TODO: Something is odd here `rand_message` takes forever.
            rand::random::<[u8; 32]>, 
            |bytes| miner.nonce(&bytes, MINIMUM_POW_SCORE),
            BatchSize::SmallInput,
        )
    });

    g.bench_function("random message (multi-threaded)", |b| {
        let miner = MinerBuilder::default().with_num_workers(num_cpus::get()).finish();
        b.iter_batched(
            rand::random::<[u8; 32]>,
            |bytes| miner.nonce(&bytes, MINIMUM_POW_SCORE),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, scoring_benchmark, mining_benchmark);
criterion_main!(benches);
