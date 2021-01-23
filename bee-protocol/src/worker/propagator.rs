// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event::MessageSolidified,
    storage::StorageBackend,
    worker::{MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent, TangleWorker},
};

use bee_message::MessageId;
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::MsTangle;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{
    any::TypeId,
    cmp::{max, min},
    convert::Infallible,
};

#[derive(Debug)]
pub(crate) struct PropagatorWorkerEvent(pub(crate) MessageId);

pub(crate) struct PropagatorWorker {
    pub(crate) tx: mpsc::UnboundedSender<PropagatorWorkerEvent>,
}

async fn propagate<B: StorageBackend>(
    message_id: MessageId,
    tangle: &MsTangle<B>,
    bus: &Bus<'static>,
    milestone_solidifier: &mpsc::UnboundedSender<MilestoneSolidifierWorkerEvent>,
) {
    let mut children = vec![message_id];

    use std::{sync::atomic::{AtomicU64, Ordering}, time::Instant};
    static TIME_TOTAL: AtomicU64 = AtomicU64::new(0);

    static ITER_TOTAL: AtomicU64 = AtomicU64::new(0);
    const SAMPLE_ITERS: u64 = 2500;

    fn start() -> Instant { Instant::now() }
    fn end(i: Instant, total: &AtomicU64, s: &str) {
        let t = total.fetch_add(i.elapsed().as_micros() as u64, Ordering::Relaxed);
        if ITER_TOTAL.load(Ordering::Relaxed) == SAMPLE_ITERS {
            println!("Avg time for {}: {} ({}% of total)", s, t as f32 / 1000.0, 100.0 * t as f32 / TIME_TOTAL.load(Ordering::Relaxed) as f32);
            total.store(0, Ordering::Relaxed);
        }
    }

    while let Some(ref message_id) = children.pop() {
        let now = std::time::Instant::now();

        if tangle.is_solid_message(message_id).await {
            continue;
        }

        // TODO Copying parents to avoid double locking, will be refactored.
        if let Some((parent1, parent2)) = tangle
            .get(&message_id)
            .await
            .map(|message| (*message.parent1(), *message.parent2()))
        {
            if !tangle.is_solid_message_maybe(&parent1).await || !tangle.is_solid_message_maybe(&parent2).await {
                continue;
            }

            // get OTRSI/YTRSI from parents
            let parent1_otsri = tangle.otrsi(&parent1).await;
            let parent2_otsri = tangle.otrsi(&parent2).await;
            let parent1_ytrsi = tangle.ytrsi(&parent1).await;
            let parent2_ytrsi = tangle.ytrsi(&parent2).await;

            // Faster than the above
            // let p1m = tangle.get_metadata(&parent1).await;
            // let p2m = tangle.get_metadata(&parent2).await;
            // let p1sepi = tangle.get_solid_entry_point_index(&parent1);
            // let p2sepi = tangle.get_solid_entry_point_index(&parent2);
            // let parent1_otsri = p1sepi.or_else(|| p1m?.otrsi());
            // let parent2_otsri = p2sepi.or_else(|| p2m?.otrsi());
            // let parent1_ytrsi = p1sepi.or_else(|| p1m?.ytrsi());
            // let parent2_ytrsi = p2sepi.or_else(|| p2m?.ytrsi());

            // get best OTRSI/YTRSI from parents
            // unwrap() is safe because parents are solid which implies that OTRSI/YTRSI values are
            // available.
            let best_otrsi = max(parent1_otsri.unwrap(), parent2_otsri.unwrap());
            let best_ytrsi = min(parent1_ytrsi.unwrap(), parent2_ytrsi.unwrap());

            let changes = tangle
                .update_metadata(&message_id, |metadata| {
                    let old_otrsi = metadata.otrsi();
                    let old_ytrsi = metadata.ytrsi();
                    if metadata.otrsi() == Some(best_otrsi)
                        && metadata.ytrsi() == Some(best_ytrsi)
                        && metadata.flags().is_solid()
                    {
                        // No new information
                        false
                    } else {
                        // OTRSI/YTRSI values need to be set before the solid flag, to ensure that the
                        // MilestoneConeUpdater is aware of all values.
                        metadata.set_otrsi(best_otrsi);
                        metadata.set_ytrsi(best_ytrsi);
                        metadata.solidify();
                        true
                    }
                })
                .await
                .expect("Failed to fetch metadata for message that should exist");

            // No changes were made to this message's data, fast exit
            if !changes {
                continue; // Is this valid?
            }

            let index = match tangle.get_metadata(&message_id).await {
                Some(meta) => {
                    if meta.flags().is_milestone() {
                        Some(meta.milestone_index())
                    } else {
                        None
                    }
                }
                None => None,
            };

            if let Some(index) = index {
                if let Err(e) = milestone_solidifier.send(MilestoneSolidifierWorkerEvent(index)) {
                    error!("Sending solidification event failed: {}.", e);
                }
            }

            if let Some(msg_children) = tangle.get_children(&message_id).await {
                for child in msg_children {
                    children.push(child);
                }
            }

            bus.dispatch(MessageSolidified(*message_id));

            tangle.insert_tip(*message_id, parent1, parent2).await;
        }

        let time = TIME_TOTAL.fetch_add(now.elapsed().as_micros() as u64, Ordering::Relaxed);
        let iter = ITER_TOTAL.fetch_add(1, Ordering::Relaxed);
        if iter == SAMPLE_ITERS {
            println!("---- Propagator body timings ----");
            println!("Iterations = {}", iter);
            println!("Time = {}us", time);
            println!("Theoretical MPS: {}", iter as f32 / (time as f32 / 1000_000.0));

            ITER_TOTAL.store(0, Ordering::Relaxed);
            TIME_TOTAL.store(0, Ordering::Relaxed);
        }
    }
}

#[async_trait]
impl<N: Node> Worker<N> for PropagatorWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>(), TypeId::of::<MilestoneSolidifierWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let milestone_solidifier = node.worker::<MilestoneSolidifierWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let bus = node.bus();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(PropagatorWorkerEvent(message_id)) = receiver.next().await {
                propagate(message_id, &tangle, &*bus, &milestone_solidifier).await;
            }

            // let (_, mut receiver) = receiver.split();
            // let receiver = receiver.get_mut();
            // let mut count: usize = 0;
            //
            // while let Ok(PropagatorWorkerEvent(message_id)) = receiver.try_recv() {
            //     propagate(message_id, &tangle, &*bus, &milestone_solidifier).await;
            //     count += 1;
            // }
            //
            // debug!("Drained {} message ids.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
