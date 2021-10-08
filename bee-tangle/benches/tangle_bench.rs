use bee_message::{Message, MessageId};
use bee_tangle::{metadata::MessageMetadata, ConflictReason, NullHooks, Tangle};
use bee_test::rand::{message::rand_message, metadata::rand_message_metadata, number::rand_number};

use criterion::*;
use rand::seq::SliceRandom;
use tokio::runtime::Runtime;

fn random_input() -> (MessageId, Message, MessageMetadata) {
    let message = rand_message();
    let id = message.id().0;

    (id, message, rand_message_metadata())
}

async fn insert(
    tangle: &Tangle<MessageMetadata, NullHooks<MessageMetadata>>,
    id: MessageId,
    message: Message,
    metadata: MessageMetadata,
) {
    tangle.insert(id, message, metadata).await;
}

async fn update_metadata(tangle: &Tangle<MessageMetadata, NullHooks<MessageMetadata>>, id: &MessageId, timestamp: u64) {
    tangle
        .update_metadata(id, |metadata| {
            metadata.set_conflict(ConflictReason::InputUtxoAlreadySpent);
            metadata.reference(timestamp);
        })
        .await;
}

fn insert_bench(c: &mut Criterion) {
    let tangle = Tangle::<MessageMetadata, NullHooks<MessageMetadata>>::default();
    let rt = Runtime::new().unwrap();

    c.bench_function("insert", |b| {
        b.to_async(&rt).iter_batched(
            || random_input(),
            |(id, message, metadata)| insert(&tangle, id, message, metadata),
            BatchSize::SmallInput,
        );
    });
}

fn update_metadata_bench(c: &mut Criterion) {
    let tangle = Tangle::<MessageMetadata, NullHooks<MessageMetadata>>::default();
    let rt = Runtime::new().unwrap();

    let data = (0..1000).map(|_| random_input());
    let mut ids = vec![];

    for (id, message, metadata) in data {
        rt.block_on(async { tangle.insert(id, message, metadata).await });
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
