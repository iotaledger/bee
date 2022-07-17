// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, convert::Infallible};

use async_trait::async_trait;
use bee_block::{payload::milestone::MilestoneIndex, BlockId};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{block_metadata::IndexId, solid_entry_point::SolidEntryPoint, Tangle, TangleWorker};
use futures::{future::FutureExt, stream::StreamExt};
use log::*;
use ref_cast::RefCast;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    event::BlockSolidified, storage::StorageBackend, MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent,
};

#[derive(Debug)]
pub(crate) struct PropagatorWorkerEvent(pub(crate) BlockId);

pub(crate) struct PropagatorWorker {
    pub(crate) tx: mpsc::UnboundedSender<PropagatorWorkerEvent>,
}

async fn propagate<B: StorageBackend>(
    block_id: BlockId,
    tangle: &Tangle<B>,
    solidified_tx: &async_channel::Sender<(BlockId, Vec<BlockId>, Option<MilestoneIndex>)>,
) {
    let mut children = vec![block_id];

    'outer: while let Some(ref block_id) = children.pop() {
        // Skip blocks that are already solid.
        if tangle.is_solid_block(block_id).await {
            continue 'outer;
        }

        if let Some(block) = tangle.get(block_id) {
            // If one of the parents is not yet solid, we skip the current block.
            for parent in block.parents().iter() {
                if !tangle.is_solid_block(parent).await {
                    continue 'outer;
                }
            }

            // Note: There are two types of solidity carriers:
            // * solid blocks (with available history)
            // * solid entry points / sep (with verified history)
            // Because we know all parents are solid, we also know that they have set OMRSI/YMRSI values, hence we can
            // simply unwrap. We also try to minimise unnecessary Tangle API calls if - for example - the parent in
            // question turns out to be a SEP.

            let mut parent_omrsis = Vec::new();
            let mut parent_ymrsis = Vec::new();

            for parent in block.parents().iter() {
                let (parent_omrsi, parent_ymrsi) = match tangle
                    .get_solid_entry_point_index(SolidEntryPoint::ref_cast(parent))
                    .await
                {
                    Some(parent_sepi) => (IndexId::new(parent_sepi, *parent), IndexId::new(parent_sepi, *parent)),
                    // SAFETY: 'unwrap' is safe, see explanation above.
                    None => tangle
                        .get_metadata(parent)
                        .map(|parent_md| {
                            parent_md
                                .omrsi_and_ymrsi()
                                .expect("solid block with unset omrsi and ymrsi")
                        })
                        .unwrap(),
                };
                parent_omrsis.push(parent_omrsi);
                parent_ymrsis.push(parent_ymrsi);
            }

            // Derive child OMRSI/YMRSI from its parents.
            // Note: The child inherits oldest (lowest) omrsi and the newest (highest) ymrsi from its parents.
            let child_omrsi = parent_omrsis.iter().min().unwrap();
            let child_ymrsi = parent_ymrsis.iter().max().unwrap();

            let ms_index_maybe = tangle
                .update_metadata(block_id, |metadata| {
                    // The child inherits the solid property from its parents.
                    metadata.mark_solid();

                    if metadata.flags().is_milestone() {
                        metadata.milestone_index()
                    } else {
                        metadata.set_omrsi_and_ymrsi(*child_omrsi, *child_ymrsi);
                        None
                    }
                })
                .expect("Failed to fetch metadata.");

            // Try to propagate as far as possible into the future.
            if let Some(block_children) = tangle.get_children(block_id) {
                for child in block_children {
                    children.push(child);
                }
            }

            // Send child to the tip pool.
            if let Err(e) = solidified_tx
                .send((*block_id, block.parents().to_vec(), ms_index_maybe))
                .await
            {
                warn!("Failed to send block to the tip pool. Cause: {:?}", e);
            }
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

        let tangle = node.resource::<Tangle<N::Backend>>();
        let bus = node.bus();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let (solidified_tx, solidified_rx) = async_channel::unbounded();

            tokio::spawn({
                let tangle = tangle.clone();

                async move {
                    while let Ok((block_id, parents, index)) = solidified_rx.recv().await {
                        bus.dispatch(BlockSolidified { block_id });

                        const SAFETY_THRESHOLD: u32 = 5; // Number of ms before eligible section of the Tangle begins

                        // NOTE: We need to decide whether we want to put this new solid block into the tip-pool.
                        // Some things to consider:
                        // 1) During synchronization we receive many non-eligible blocks, that are way too old for
                        //    the TSA, hence we want to exclude them.
                        // 2) We don't know the confirming milestone index of each eventually confirmed block at this
                        // point in time, hence we need to employ a heuristic with a security threshold to minimise the
                        // risk of a false-negative (something excluded from the tip-pool, that would be eligible as
                        // a tip). That heuristic is: Do not add to the tip-pool as long as the Tangle is not within
                        // BELOW_MAX_DEPTH + <some_safety_threshold>
                        // 3) Even if this method doesn't work perfectly, the tip set would only be a little
                        // bit smaller than it could be for a very short time after the node is synchronized.

                        // NOTE: That diff is always okay, because of the invariant: SMI <= LMI or 0 <= (LMI - SMI)
                        if (tangle.get_latest_milestone_index() - tangle.get_solid_milestone_index())
                            <= (tangle.config().below_max_depth() + SAFETY_THRESHOLD).into()
                        {
                            tangle.insert_tip(block_id, parents).await;
                        }

                        if let Some(index) = index {
                            if let Err(e) = milestone_solidifier.send(MilestoneSolidifierWorkerEvent(index)) {
                                error!("Sending solidification event failed: {}.", e);
                            }
                        }
                    }
                }
            });

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(PropagatorWorkerEvent(block_id)) = receiver.next().await {
                propagate(block_id, &tangle, &solidified_tx).await;
            }

            // Before the worker completely stops, the receiver needs to be drained for statuses to be propagated.
            // Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(PropagatorWorkerEvent(block_id))) = receiver.next().now_or_never() {
                propagate(block_id, &tangle, &solidified_tx).await;
                count += 1;
            }

            debug!("Drained {} events.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
