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
use log::{debug, error, info};
use tokio::sync::mpsc;

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

    while let Some(ref message_id) = children.pop() {
        if tangle.is_solid_message(message_id).await {
            continue;
        }

        // TODO Copying parents to avoid double locking, will be refactored.
        if let Some((parent1, parent2)) = tangle
            .get(&message_id)
            .await
            .map(|message| (*message.parent1(), *message.parent2()))
        {
            if !tangle.is_solid_message(&parent1).await || !tangle.is_solid_message(&parent2).await {
                continue;
            }

            // get OTRSI/YTRSI from parents
            let parent1_otsri = tangle.otrsi(&parent1).await;
            let parent2_otsri = tangle.otrsi(&parent2).await;
            let parent1_ytrsi = tangle.ytrsi(&parent1).await;
            let parent2_ytrsi = tangle.ytrsi(&parent2).await;

            // get best OTRSI/YTRSI from parents
            // unwrap() is safe because parents are solid which implies that OTRSI/YTRSI values are
            // available.
            let best_otrsi = max(parent1_otsri.unwrap(), parent2_otsri.unwrap());
            let best_ytrsi = min(parent1_ytrsi.unwrap(), parent2_ytrsi.unwrap());

            tangle
                .update_metadata(&message_id, |metadata| {
                    // OTRSI/YTRSI values need to be set before the solid flag, to ensure that the
                    // MilestoneConeUpdater is aware of all values.
                    metadata.set_otrsi(best_otrsi);
                    metadata.set_ytrsi(best_ytrsi);
                    metadata.solidify();

                    if metadata.flags().is_milestone() {
                        if let Err(e) =
                            milestone_solidifier.send(MilestoneSolidifierWorkerEvent(metadata.milestone_index()))
                        {
                            error!("Sending solidification event failed: {}.", e);
                        }
                    }
                })
                .await;

            if let Some(msg_children) = tangle.get_children(&message_id).await {
                for child in msg_children {
                    children.push(child);
                }
            }

            bus.dispatch(MessageSolidified(*message_id));

            tangle.insert_tip(*message_id, parent1, parent2).await;
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

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(PropagatorWorkerEvent(message_id)) = receiver.next().await {
                propagate(message_id, &tangle, &*bus, &milestone_solidifier).await;
            }

            let (_, mut receiver) = receiver.split();
            let receiver = receiver.get_mut();
            let mut count = 0;

            while let Ok(PropagatorWorkerEvent(message_id)) = receiver.try_recv() {
                propagate(message_id, &tangle, &*bus, &milestone_solidifier).await;
                count += 1;
            }

            debug!("Drained {} message ids.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
