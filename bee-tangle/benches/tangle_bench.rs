// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    rand::{block::rand_block, number::rand_number},
    semantic::ConflictReason,
    Block, BlockId,
};
use bee_runtime::resource::ResourceHandle;
use bee_storage_null::Storage as NullStorage;
use bee_tangle::{
    block_metadata::BlockMetadata, config::TangleConfig, rand::block_metadata::rand_block_metadata, Tangle,
};
use criterion::*;
use rand::seq::SliceRandom;

fn random_input() -> (Block, BlockId, BlockMetadata) {
    let block = rand_block();
    let id = block.id();

    (block, id, rand_block_metadata())
}

fn update_metadata(tangle: &Tangle<NullStorage>, id: &BlockId, timestamp: u32) {
    tangle.update_metadata(id, |metadata| {
        metadata.set_conflict(ConflictReason::InputUtxoAlreadySpent);
        metadata.reference(timestamp);
    });
}

fn insert_bench(c: &mut Criterion) {
    let storage = ResourceHandle::<NullStorage>::new(NullStorage);
    let config = TangleConfig::build().finish();
    let tangle = Tangle::new(config, storage);

    c.bench_function("insert", |b| {
        b.iter_batched(
            random_input,
            |(block, id, metadata)| tangle.insert(&block, &id, &metadata),
            BatchSize::SmallInput,
        );
    });
}

fn update_metadata_bench(c: &mut Criterion) {
    let storage = ResourceHandle::<NullStorage>::new(NullStorage);
    let config = TangleConfig::build().finish();
    let tangle = Tangle::new(config, storage);

    let data = (0..1000).map(|_| random_input());
    let mut ids = vec![];

    for (block, id, metadata) in data {
        tangle.insert(&block, &id, &metadata);
        ids.push(id);
    }

    c.bench_function("update_metadata", |b| {
        b.iter_batched(
            || (ids.choose(&mut rand::thread_rng()).unwrap(), rand_number::<u32>()),
            |(id, timestamp)| update_metadata(&tangle, id, timestamp),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, insert_bench, update_metadata_bench);
criterion_main!(benches);
