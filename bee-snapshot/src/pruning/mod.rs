// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod config;

pub use config::{PruningConfig, PruningConfigBuilder};

// use crate::constants::{ADDITIONAL_PRUNING_THRESHOLD, SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST};

// use bee_protocol::{
//     tangle::{helper, MsTangle},
//     MilestoneIndex,
// };
// use bee_tangle::traversal;
// use bee_storage::storage::Backend;

// use dashmap::DashMap;

// use log::{info, warn};

// #[derive(Debug)]
// pub enum Error {
//     MilestoneNotFoundInTangle(u32),
//     MetadataNotFound(Box<Hash>),
// }

// /// Checks whether any direct approver of the given transaction was confirmed by a
// /// milestone which is above the target milestone.
// pub fn is_solid_entry_point<B: Backend>(tangle: &MsTangle<B>, hash: &Hash) -> Result<bool, Error> {
//     // Check if there is any approver of the transaction was confirmed by newer milestones.
//     let milestone_index;
//     if let Some(metadata) = tangle.get_metadata(hash) {
//         milestone_index = metadata.milestone_index();
//     } else {
//         return Err(Error::MetadataNotFound(Box::new(*hash)));
//     }
//     let mut is_solid = false;
//     traversal::visit_children_follow_parent1(
//         tangle,
//         *hash,
//         |_, metadata| {
//             if is_solid {
//                 return false;
//             }
//             // `true` when one of the current tx's approver was confirmed by a newer milestone_index.
//             is_solid = metadata.flags().is_confirmed() && metadata.milestone_index() > milestone_index;
//             true
//         },
//         |_, _, _| {},
//     );
//     Ok(is_solid)
// }

// // TODO testing
// pub fn get_new_solid_entry_points<B: Backend>(tangle: &MsTangle<B>, target_index: MilestoneIndex) ->
// Result<DashMap<Hash, MilestoneIndex>, Error> {     let solid_entry_points = DashMap::<Hash, MilestoneIndex>::new();
//     for index in *target_index - SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST..*target_index {
//         let milestone_hash;

//         // NOTE Actually we don't really need the tail, and only need one of the milestone tx.
//         //      In gohornet, we start from the tail milestone tx.
//         if let Some(hash) = tangle.get_milestone_hash(MilestoneIndex(index)) {
//             milestone_hash = hash;
//         } else {
//             return Err(Error::MilestoneNotFoundInTangle(index));
//         }

//         // Get all the approvees confirmed by the milestone tail.
//         traversal::visit_parents_depth_first(
//             tangle,
//             milestone_hash,
//             |_, _, metadata| *metadata.milestone_index() >= index,
//             |hash, _, metadata| {
//                 if metadata.flags().is_confirmed() && is_solid_entry_point(tangle, &hash).unwrap() {
//                     // Find all tails.
//                     helper::on_all_tails(&**tangle, *hash, |hash, _tx, metadata| {
//                         solid_entry_points.insert(*hash, metadata.milestone_index());
//                     });
//                 }
//             },
//             |_, _, _| {},
//             |_| {},
//         );
//     }
//     Ok(solid_entry_points)
// }

// #[allow(dead_code)]
// // TODO get the unconfirmed transactions in the database.
// pub fn get_unconfirmed_transactions(_target_index: &MilestoneIndex) -> Vec<Hash> {
//     // NOTE If there is no specific struct for storing th unconfirmed transaction,
//     //      then we need to traverse the whole tangle to get the unconfirmed transactions (SLOW)!
//     // TODO traverse the whole tangle through the approvers from solid entry points.
//     unimplemented!()
// }

// // TODO remove the unconfirmed transactions in the database.
// pub fn prune_unconfirmed_transactions(_purning_milestone_index: &MilestoneIndex) -> u32 {
//     unimplemented!()
// }

// // TODO remove the confirmed transactions in the database.
// pub fn prune_transactions(_hashes: Vec<Hash>) -> u32 {
//     unimplemented!()
// }

// // TODO prunes the milestone metadata and the ledger diffs from the database for the given milestone.
// pub fn prune_milestone(_milestone_index: MilestoneIndex) {
//     // Delete ledger_diff for milestone with milestone_index.
//     // Delete milestone storage (if we have this) for milestone with milestone_index.
//     unimplemented!()
// }

// // NOTE we don't prune cache, but only prune the database.
// pub fn prune_database<B: Backend>(tangle: &MsTangle<B>, mut target_index: MilestoneIndex) -> Result<(), Error> {
//     let target_index_max = MilestoneIndex(
//         *tangle.get_snapshot_index() - SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST - ADDITIONAL_PRUNING_THRESHOLD - 1,
//     );
//     if target_index > target_index_max {
//         target_index = target_index_max;
//     }
//     // Update the solid entry points in the static MsTangle.
//     let new_solid_entry_points = get_new_solid_entry_points(tangle, target_index)?;

//     // Clear the solid_entry_points in the static MsTangle.
//     tangle.clear_solid_entry_points();

//     // TODO update the whole solid_entry_points in the static MsTangle w/o looping.
//     for (hash, milestone_index) in new_solid_entry_points.into_iter() {
//         tangle.add_solid_entry_point(hash, milestone_index);
//     }

//     // We have to set the new solid entry point index.
//     // This way we can cleanly prune even if the pruning was aborted last time.
//     tangle.update_entry_point_index(target_index);

//     prune_unconfirmed_transactions(&tangle.get_pruning_index());

//     // Iterate through all milestones that have to be pruned.
//     for milestone_index in *tangle.get_pruning_index() + 1..*target_index + 1 {
//         info!("Pruning milestone {}...", milestone_index);

//         // TODO calculate the deleted tx count and visited tx count if needed
//         let pruned_unconfirmed_transactions_count = prune_unconfirmed_transactions(&MilestoneIndex(milestone_index));

//         // NOTE Actually we don't really need the tail, and only need one of the milestone tx.
//         //      In gohornet, we start from the tail milestone tx.
//         let milestone_hash;
//         if let Some(hash) = tangle.get_milestone_hash(MilestoneIndex(milestone_index)) {
//             milestone_hash = hash;
//         } else {
//             warn!("Pruning milestone {} failed! Milestone not found!", milestone_index);
//             continue;
//         }

//         let mut transactions_to_prune: Vec<Hash> = Vec::new();

//         // Traverse the approvees of the milestone transaction.
//         traversal::visit_parents_depth_first(
//             tangle,
//             milestone_hash,
//             |_, _, _| {
//                 // NOTE everything that was referenced by that milestone can be pruned
//                 //      (even transactions of older milestones)
//                 true
//             },
//             |hash, _, _| transactions_to_prune.push(*hash),
//             |_, _, _| {},
//             |_| {},
//         );

//         // NOTE The metadata of solid entry points can be deleted from the database,
//         //      because we only need the hashes of them, and don't have to trace their parents.
//         let transactions_to_prune_count = transactions_to_prune.len();
//         let pruned_transactions_count = prune_transactions(transactions_to_prune);

//         prune_milestone(MilestoneIndex(milestone_index));

//         tangle.update_pruning_index(MilestoneIndex(milestone_index));
//         info!(
//             "Pruning milestone {}. Pruned {}/{} confirmed transactions. Pruned {} unconfirmed transactions.",
//             milestone_index,
//             pruned_transactions_count,
//             transactions_to_prune_count,
//             pruned_unconfirmed_transactions_count
//         );
//         // TODO trigger pruning milestone index changed event if needed.
//         //      notify peers about our new pruning milestone index by
//         //      broadcast_heartbeat()
//     }
//     Ok(())
// }
