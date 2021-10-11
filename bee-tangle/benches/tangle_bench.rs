// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId};
use bee_runtime::resource::ResourceHandle;
use bee_storage_null::Storage as NullStorage;
use bee_tangle::{config::TangleConfig, metadata::MessageMetadata, ConflictReason, Tangle};
use bee_test::rand::{message::rand_message, metadata::rand_message_metadata, number::rand_number};

use criterion::*;
use rand::seq::SliceRandom;
use tokio::runtime::Runtime;

fn random_input() -> (Message, MessageId, MessageMetadata) {
    let message = rand_message();
    let id = message.id().0;

    (message, id, rand_message_metadata())
}

async fn insert(tangle: &Tangle<NullStorage>, message: Message, id: MessageId, metadata: MessageMetadata) {
    tangle.insert(message, id, metadata).await;
}

async fn update_metadata(tangle: &Tangle<NullStorage>, id: &MessageId, timestamp: u64) {
    tangle
        .update_metadata(id, |metadata| {
            metadata.set_conflict(ConflictReason::InputUtxoAlreadySpent);
            metadata.reference(timestamp);
        })
        .await;
}

fn insert_bench(c: &mut Criterion) {
    let storage = ResourceHandle::<NullStorage>::new(NullStorage);
    let config = TangleConfig::build().finish();
    let tangle = Tangle::new(config, storage);
    let rt = Runtime::new().unwrap();

    c.bench_function("insert", |b| {
        b.to_async(&rt).iter_batched(
            random_input,
            |(message, id, metadata)| insert(&tangle, message, id, metadata),
            BatchSize::SmallInput,
        );
    });
}

fn update_metadata_bench(c: &mut Criterion) {
    let storage = ResourceHandle::<NullStorage>::new(NullStorage);
    let config = TangleConfig::build().finish();
    let tangle = Tangle::new(config, storage);
    let rt = Runtime::new().unwrap();

    let data = (0..1000).map(|_| random_input());
    let mut ids = vec![];

    for (message, id, metadata) in data {
        rt.block_on(async { tangle.insert(message, id, metadata).await });
        ids.push(id);
    }

    c.bench_function("update_metadata", |b| {
        b.to_async(&rt).iter_batched(
            || (ids.choose(&mut rand::thread_rng()).unwrap(), rand_number::<u64>()),
            |(id, timestamp)| update_metadata(&tangle, id, timestamp),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, insert_bench, update_metadata_bench);
criterion_main!(benches);
