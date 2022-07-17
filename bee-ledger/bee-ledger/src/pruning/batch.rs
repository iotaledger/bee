// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

use bee_block::{output::OutputId, payload::milestone::MilestoneIndex, Block, BlockId};
use bee_storage::access::{Batch, Fetch};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock, Tangle,
};
use hashbrown::{HashMap, HashSet};
use ref_cast::RefCast;

use crate::{
    consensus::worker::EXTRA_PRUNING_DEPTH,
    pruning::{
        error::Error,
        metrics::{ConfirmedDataPruningMetrics, MilestoneDataPruningMetrics, UnconfirmedDataPruningMetrics},
    },
    storage::StorageBackend,
    types::{ConsumedOutput, CreatedOutput, OutputDiff, Receipt},
};

pub type Blocks = HashSet<BlockId>;
pub type ApproverCache = HashMap<BlockId, MilestoneIndex>;
pub type Seps = HashMap<SolidEntryPoint, MilestoneIndex>;

#[derive(Eq, PartialEq, Hash)]
pub struct Edge {
    pub from_parent: BlockId,
    pub to_child: BlockId,
}

pub fn prune_confirmed_data<S: StorageBackend>(
    tangle: &Tangle<S>,
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
    current_seps: &Seps,
) -> Result<(Seps, ConfirmedDataPruningMetrics), Error> {
    // We keep a list of already visited blocks.
    let mut visited = Blocks::with_capacity(512);
    // We keep a cache of approvers to prevent fetch the same data from the storage more than once.
    let mut approver_cache = ApproverCache::with_capacity(512);
    // We collect new SEPs during the traversal, and return them as a result of this function.
    let mut new_seps = Seps::with_capacity(512);
    // We collect stats during the traversal, and return them as a result of this function.
    let mut metrics = ConfirmedDataPruningMetrics::default();
    // FIXME: mitigation code
    let mitigation_threshold = tangle.config().below_max_depth() + EXTRA_PRUNING_DEPTH; // = BMD + 5

    // Get the `BlockId` of the milestone we are about to prune from the storage.
    let prune_id = *Fetch::<MilestoneIndex, MilestoneMetadata>::fetch(storage, &prune_index)
        .map_err(|e| Error::Storage(Box::new(e)))?
        .ok_or(Error::MissingMilestone(prune_index))?
        .block_id();

    // Breadth-first traversal will increase our chances of sorting out redundant blocks without querying the storage.
    let mut to_visit: VecDeque<_> = vec![prune_id].into_iter().collect();

    while let Some(block_id) = to_visit.pop_front() {
        // Skip already visited blocks.
        if visited.contains(&block_id) {
            metrics.block_already_visited += 1;
            continue;
        }

        // Skip SEPs (from the previous pruning run).
        if current_seps.contains_key(SolidEntryPoint::ref_cast(&block_id)) {
            metrics.references_sep += 1;
            continue;
        }

        // Get the `Block` for `block_id`.
        let block = match Fetch::<BlockId, Block>::fetch(storage, &block_id)
            .map_err(|e| Error::Storage(Box::new(e)))?
            .ok_or(Error::MissingBlock(block_id))
        {
            Ok(block) => block,
            Err(e) => {
                // Note: if we land here, then one of those things can have happened:
                // (a) the storage has been messed with, and is therefore faulty,
                // (b) the algo didn't turn a confirmed block into an SEP although it should have (bug),
                // (c) the algo treated a in fact confirmed block as unconfirmed, and removed it (bug).
                log::error!(
                    "failed to fetch `Block` associated with block id {} during past-cone traversal of milestone {} ({})",
                    &block_id,
                    &prune_index,
                    &prune_id,
                );

                return Err(e);
            }
        };

        // Delete its edges.
        let parents = block.parents();
        for parent_id in parents.iter() {
            prune_edge(storage, batch, &(*parent_id, block_id))?;
            metrics.prunable_edges += 1;
        }

        // Add its parents to the queue of yet to traverse blocks.
        to_visit.extend(block.into_parents().iter());

        // Remember that we've seen this block already.
        visited.insert(block_id);

        // Delete its associated data.
        prune_block_and_metadata(storage, batch, &block_id)?;

        // ---
        // Everything that follows is required to decide whether this block's id should be kept as a solid entry
        // point. We keep the set of SEPs minimal by checking whether there are still blocks in future
        // milestone cones (beyond the current target index) that are referencing the currently processed
        // block (similar to a garbage collector we remove objects only if nothing is referencing it anymore).
        // ---

        // Fetch its approvers from the storage.
        let approvers = Fetch::<BlockId, Vec<BlockId>>::fetch(storage, &block_id)
            .map_err(|e| Error::Storage(Box::new(e)))?
            .ok_or(Error::MissingApprovers(block_id))?;

        // We can safely skip blocks whose approvers are all part of the currently pruned cone. If we are lucky
        // (chances are better with the chosen breadth-first traversal) we've already seen all of its approvers.
        let mut unvisited_approvers = approvers.into_iter().filter(|id| !visited.contains(id)).peekable();
        if unvisited_approvers.peek().is_none() {
            metrics.all_approvers_visited += 1;
            continue;
        }

        metrics.not_all_approvers_visited += 1;

        // To decide for how long we need to keep a particular SEP around, we need to know the greatest confirming index
        // taken over all its approvers. We initialise this value with the lowest possible value (the current pruning
        // target index).
        let mut max_conf_index = *prune_index;

        for unvisited_id in unvisited_approvers {
            let approver_conf_index = if let Some(conf_index) = approver_cache.get(&unvisited_id) {
                // We fetched the metadata of this approver before (fast path).
                metrics.approver_cache_hit += 1;

                **conf_index
            } else {
                // We need to fetch the metadata of this approver (slow path).
                metrics.approver_cache_miss += 1;

                let unvisited_md = Fetch::<BlockId, BlockMetadata>::fetch(storage, &unvisited_id)
                    .map_err(|e| Error::Storage(Box::new(e)))?
                    .ok_or(Error::MissingMetadata(unvisited_id))?;

                // Note, that an unvisited approver of this block can still be confirmed by the same milestone
                // (despite the breadth-first traversal), if it is also its sibling.
                let conf_index = unvisited_md.milestone_index().unwrap_or_else(|| {
                    // ---
                    // BUG/FIXME:
                    // In very rare situations the milestone index has not been set for a confirmed block. If that
                    // block happens to be the one with the highest confirmation index, then the SEP created from the
                    // current block would be removed too early, i.e. before all of its referrers, and pruning would
                    // fail without a way to ever recover. We suspect the bug to be a race condition in the
                    // `update_metadata` method of the `Tangle` implementation.
                    //
                    // Mitigation strategy:
                    // We rely on the coordinator to not confirm something that attaches to a block that was confirmed
                    // more than 20 milestones (BMD + EXTRA_PRUNING_DEPTH) ago, i.e. a lazy tip.
                    // ---
                    log::trace!(
                        "Bug mitigation: Using '{} + mitigation_threshold ({})' for approver '{}'",
                        prune_index,
                        mitigation_threshold,
                        &unvisited_id
                    );

                    prune_index + mitigation_threshold
                });

                // Update the approver cache.
                approver_cache.insert(unvisited_id, conf_index);

                *conf_index
            };

            max_conf_index = max_conf_index.max(approver_conf_index);
        }

        // If the highest confirmation index of all its approvers is greater than the index we're pruning, then we need
        // to keep its block id as a solid entry point.
        if max_conf_index > *prune_index {
            new_seps.insert(block_id.into(), max_conf_index.into());

            log::trace!("New SEP: {} until {}", block_id, max_conf_index);

            metrics.found_seps += 1;
        }
    }

    metrics.prunable_blocks = visited.len();
    metrics.new_seps = new_seps.len();

    Ok((new_seps, metrics))
}

pub fn prune_unconfirmed_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
) -> Result<UnconfirmedDataPruningMetrics, Error> {
    let mut metrics = UnconfirmedDataPruningMetrics::default();

    let unconf_blocks = match Fetch::<MilestoneIndex, Vec<UnreferencedBlock>>::fetch(storage, &prune_index)
        .map_err(|e| Error::Storage(Box::new(e)))?
    {
        Some(unconf_blocks) => {
            if unconf_blocks.is_empty() {
                metrics.none_received = true;
                Vec::new()
            } else {
                unconf_blocks
            }
        }
        None => {
            metrics.none_received = true;
            Vec::new()
        }
    };

    // TODO: consider using `MultiFetch`
    'outer_loop: for unconf_block_id in unconf_blocks.iter().map(|unconf_block| unconf_block.block_id()) {
        // Skip those that were confirmed.
        match Fetch::<BlockId, BlockMetadata>::fetch(storage, unconf_block_id)
            .map_err(|e| Error::Storage(Box::new(e)))?
        {
            Some(block_meta) => {
                if block_meta.flags().is_referenced() {
                    metrics.were_confirmed += 1;
                    continue;
                } else {
                    // We log which blocks were never confirmed.
                    log::trace!("'referenced' flag not set for {}", unconf_block_id);

                    // ---
                    // BUG/FIXME:
                    // In very rare situations the `referenced` flag has not been set for a confirmed block. This
                    // would lead to it being removed as an unconfirmed block causing the past-cone traversal of a
                    // milestone to fail. That would cause pruning to fail without a way to ever recover. We suspect the
                    // bug to be a race condition in the `update_metadata` method of the `Tangle` implementation.
                    //
                    // Mitigation strategy:
                    // To make occurring this scenario sufficiently unlikely, we only prune a block with
                    // the flag indicating "not referenced", if all its approvers are also flagged as "not referenced".
                    // In other words: If we find at least one confirmed approver, then we know the flag wasn't set
                    // appropriatedly for the current block due to THE bug, and that we cannot prune it.
                    // ---
                    let unconf_approvers = Fetch::<BlockId, Vec<BlockId>>::fetch(storage, unconf_block_id)
                        .map_err(|e| Error::Storage(Box::new(e)))?
                        .ok_or(Error::MissingApprovers(*unconf_block_id))?;

                    for unconf_approver_id in unconf_approvers {
                        if let Some(unconf_approver_md) =
                            Fetch::<BlockId, BlockMetadata>::fetch(storage, &unconf_approver_id)
                                .map_err(|e| Error::Storage(Box::new(e)))?
                        {
                            if unconf_approver_md.flags().is_referenced() {
                                continue 'outer_loop;
                            }
                        }
                    }

                    log::trace!("all of '{}'s approvers are flagged 'unreferenced'", unconf_block_id);
                }
            }
            None => {
                metrics.already_pruned += 1;
                continue;
            }
        }

        // Delete those blocks that remained unconfirmed.
        match Fetch::<BlockId, Block>::fetch(storage, unconf_block_id).map_err(|e| Error::Storage(Box::new(e)))? {
            Some(block) => {
                let parents = block.parents();

                // Add block data to the delete batch.
                prune_block_and_metadata(storage, batch, unconf_block_id)?;

                log::trace!("Pruned unconfirmed block {} at {}.", unconf_block_id, prune_index);

                // Add prunable edges to the delete batch.
                for parent in parents.iter() {
                    prune_edge(storage, batch, &(*parent, *unconf_block_id))?;

                    metrics.prunable_edges += 1;
                }
            }
            None => {
                metrics.already_pruned += 1;
                continue;
            }
        }

        Batch::<(MilestoneIndex, UnreferencedBlock), ()>::batch_delete(
            storage,
            batch,
            &(prune_index, (*unconf_block_id).into()),
        )
        .map_err(|e| Error::Storage(Box::new(e)))?;

        metrics.prunable_blocks += 1;
    }

    Ok(metrics)
}

pub fn prune_milestone_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
    should_prune_receipts: bool,
) -> Result<MilestoneDataPruningMetrics, Error> {
    let mut metrics = MilestoneDataPruningMetrics::default();

    prune_milestone(storage, batch, prune_index)?;

    prune_output_diff(storage, batch, prune_index)?;

    if should_prune_receipts {
        metrics.receipts = prune_receipts(storage, batch, prune_index)?;
    }

    Ok(metrics)
}

fn prune_block_and_metadata<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    block_id: &BlockId,
) -> Result<(), Error> {
    Batch::<BlockId, Block>::batch_delete(storage, batch, block_id).map_err(|e| Error::Storage(Box::new(e)))?;
    Batch::<BlockId, BlockMetadata>::batch_delete(storage, batch, block_id).map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

fn prune_edge<S: StorageBackend>(storage: &S, batch: &mut S::Batch, edge: &(BlockId, BlockId)) -> Result<(), Error> {
    Batch::<(BlockId, BlockId), ()>::batch_delete(storage, batch, edge).map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

fn prune_milestone<S: StorageBackend>(storage: &S, batch: &mut S::Batch, index: MilestoneIndex) -> Result<(), Error> {
    Batch::<MilestoneIndex, MilestoneMetadata>::batch_delete(storage, batch, &index)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

fn prune_output_diff<S: StorageBackend>(storage: &S, batch: &mut S::Batch, index: MilestoneIndex) -> Result<(), Error> {
    if let Some(output_diff) =
        Fetch::<MilestoneIndex, OutputDiff>::fetch(storage, &index).map_err(|e| Error::Storage(Box::new(e)))?
    {
        for consumed_output in output_diff.consumed_outputs() {
            Batch::<OutputId, ConsumedOutput>::batch_delete(storage, batch, consumed_output)
                .map_err(|e| Error::Storage(Box::new(e)))?;
            Batch::<OutputId, CreatedOutput>::batch_delete(storage, batch, consumed_output)
                .map_err(|e| Error::Storage(Box::new(e)))?;
        }

        if let Some(_treasury_diff) = output_diff.treasury_diff() {
            // TODO
        }
    }

    Batch::<MilestoneIndex, OutputDiff>::batch_delete(storage, batch, &index)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

fn prune_receipts<S: StorageBackend>(storage: &S, batch: &mut S::Batch, index: MilestoneIndex) -> Result<usize, Error> {
    let receipts = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(storage, &index)
        .map_err(|e| Error::Storage(Box::new(e)))?
        // Fine since Fetch of a Vec<_> always returns Some(Vec<_>).
        .unwrap();

    let mut num = 0;
    for receipt in receipts.into_iter() {
        Batch::<(MilestoneIndex, Receipt), ()>::batch_delete(storage, batch, &(index, receipt))
            .map_err(|e| Error::Storage(Box::new(e)))?;

        num += 1;
    }

    Ok(num)
}

// TODO: consider using this instead of 'truncate'
#[allow(dead_code)]
fn prune_seps<S: StorageBackend>(storage: &S, batch: &mut S::Batch, seps: &[SolidEntryPoint]) -> Result<usize, Error> {
    let mut num = 0;
    for sep in seps {
        Batch::<SolidEntryPoint, MilestoneIndex>::batch_delete(storage, batch, sep)
            .map_err(|e| Error::Storage(Box::new(e)))?;

        num += 1;
    }

    Ok(num)
}
