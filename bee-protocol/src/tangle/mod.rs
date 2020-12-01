// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod metadata;
mod urts;

pub mod flags;

pub use metadata::MessageMetadata;

use crate::{
    milestone::MilestoneIndex,
    tangle::{flags::Flags, urts::UrtsTipPool},
};

use bee_common::node::ResHandle;
use bee_message::{Message, MessageId};
use bee_storage::storage::Backend;
use bee_tangle::{Hooks, MessageRef, Tangle};

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::Mutex;

use std::{
    ops::Deref,
    sync::atomic::{AtomicU32, Ordering},
};

pub struct StorageHooks<B> {
    #[allow(dead_code)]
    storage: ResHandle<B>,
}

#[async_trait]
impl<B: Backend> Hooks<MessageMetadata> for StorageHooks<B> {
    type Error = ();

    async fn get(&self, _hash: &MessageId) -> Result<(Message, MessageMetadata), Self::Error> {
        // println!("Attempted to fetch {:?} from storage", hash);
        Err(())
    }

    async fn insert(&self, _hash: MessageId, _tx: Message, _metadata: MessageMetadata) -> Result<(), Self::Error> {
        // println!("Attempted to insert {:?} into storage", hash);
        Ok(())
    }
}

/// Milestone-based Tangle.
pub struct MsTangle<B> {
    pub(crate) inner: Tangle<MessageMetadata, StorageHooks<B>>,
    pub(crate) milestones: DashMap<MilestoneIndex, MessageId>,
    pub(crate) solid_entry_points: DashMap<MessageId, MilestoneIndex>,
    latest_milestone_index: AtomicU32,
    latest_solid_milestone_index: AtomicU32,
    snapshot_index: AtomicU32,
    pruning_index: AtomicU32,
    entry_point_index: AtomicU32,
    tip_pool: Mutex<UrtsTipPool>,
}

impl<B> Deref for MsTangle<B> {
    type Target = Tangle<MessageMetadata, StorageHooks<B>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<B: Backend> MsTangle<B> {
    pub fn new(storage: ResHandle<B>) -> Self {
        Self {
            inner: Tangle::new(StorageHooks { storage }),
            milestones: Default::default(),
            solid_entry_points: Default::default(),
            latest_milestone_index: Default::default(),
            latest_solid_milestone_index: Default::default(),
            snapshot_index: Default::default(),
            pruning_index: Default::default(),
            entry_point_index: Default::default(),
            tip_pool: Mutex::new(UrtsTipPool::default()),
        }
    }

    pub async fn shutdown(self) {
        // TODO: Write back changes by calling self.inner.shutdown().await
    }

    pub async fn insert(&self, message: Message, hash: MessageId, metadata: MessageMetadata) -> Option<MessageRef> {
        // TODO this has been temporarily moved to the processor.
        // Reason is that since the tangle is not a worker, it can't have access to the propagator tx.
        // When the tangle is made a worker, this should be put back on.
        // if opt.is_some() {
        //     if let Err(e) = Protocol::get()
        //         .propagator_worker
        //         .unbounded_send(PropagatorWorkerEvent(hash))
        //     {
        //         error!("Failed to send hash to propagator: {:?}.", e);
        //     }
        // }
        //
        // opt
        self.inner.insert(hash, message, metadata).await
    }

    pub fn add_milestone(&self, index: MilestoneIndex, hash: MessageId) {
        // TODO: only insert if vacant
        self.milestones.insert(index, hash);
        self.inner.update_metadata(&hash, |metadata| {
            metadata.flags_mut().set_milestone(true);
            metadata.set_milestone_index(index);
        });
    }

    pub fn remove_milestone(&self, index: MilestoneIndex) {
        self.milestones.remove(&index);
    }

    // TODO: use combinator instead of match
    pub async fn get_milestone(&self, index: MilestoneIndex) -> Option<MessageRef> {
        match self.get_milestone_message_id(index) {
            None => None,
            Some(ref hash) => self.get(hash).await,
        }
    }

    // TODO: use combinator instead of match
    pub fn get_milestone_message_id(&self, index: MilestoneIndex) -> Option<MessageId> {
        match self.milestones.get(&index) {
            None => None,
            Some(v) => Some(*v),
        }
    }

    pub fn contains_milestone(&self, index: MilestoneIndex) -> bool {
        self.milestones.contains_key(&index)
    }

    pub fn get_latest_milestone_index(&self) -> MilestoneIndex {
        self.latest_milestone_index.load(Ordering::Relaxed).into()
    }

    pub fn update_latest_milestone_index(&self, new_index: MilestoneIndex) {
        self.latest_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    pub fn get_latest_solid_milestone_index(&self) -> MilestoneIndex {
        self.latest_solid_milestone_index.load(Ordering::Relaxed).into()
    }

    pub fn update_latest_solid_milestone_index(&self, new_index: MilestoneIndex) {
        self.latest_solid_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    pub fn get_snapshot_index(&self) -> MilestoneIndex {
        self.snapshot_index.load(Ordering::Relaxed).into()
    }

    pub fn update_snapshot_index(&self, new_index: MilestoneIndex) {
        self.snapshot_index.store(*new_index, Ordering::Relaxed);
    }

    pub fn get_pruning_index(&self) -> MilestoneIndex {
        self.pruning_index.load(Ordering::Relaxed).into()
    }

    pub fn update_pruning_index(&self, new_index: MilestoneIndex) {
        self.pruning_index.store(*new_index, Ordering::Relaxed);
    }

    pub fn get_entry_point_index(&self) -> MilestoneIndex {
        self.entry_point_index.load(Ordering::Relaxed).into()
    }

    pub fn update_entry_point_index(&self, new_index: MilestoneIndex) {
        self.entry_point_index.store(*new_index, Ordering::Relaxed);
    }

    // TODO reduce to one atomic value ?
    pub fn is_synced(&self) -> bool {
        self.is_synced_threshold(0)
    }

    // TODO reduce to one atomic value ?
    pub fn is_synced_threshold(&self, threshold: u32) -> bool {
        *self.get_latest_solid_milestone_index() >= (*self.get_latest_milestone_index() - threshold)
    }

    pub fn get_solid_entry_point_index(&self, hash: &MessageId) -> Option<MilestoneIndex> {
        self.solid_entry_points.get(hash).map(|i| *i)
    }

    pub fn add_solid_entry_point(&self, hash: MessageId, index: MilestoneIndex) {
        self.solid_entry_points.insert(hash, index);
    }

    /// Removes `hash` from the set of solid entry points.
    pub fn remove_solid_entry_point(&self, hash: &MessageId) {
        self.solid_entry_points.remove(hash);
    }

    pub fn clear_solid_entry_points(&self) {
        self.solid_entry_points.clear();
    }

    /// Returns whether the message associated with `hash` is a solid entry point.
    pub fn is_solid_entry_point(&self, hash: &MessageId) -> bool {
        self.solid_entry_points.contains_key(hash)
    }

    /// Returns whether the message associated with `hash` is deemed `solid`.
    pub fn is_solid_message(&self, hash: &MessageId) -> bool {
        if self.is_solid_entry_point(hash) {
            true
        } else {
            self.inner
                .get_metadata(hash)
                .map(|metadata| metadata.flags().is_solid())
                .unwrap_or(false)
        }
    }

    pub fn otrsi(&self, hash: &MessageId) -> Option<MilestoneIndex> {
        match self.solid_entry_points.get(hash) {
            Some(sep) => Some(*sep.value()),
            None => match self.get_metadata(hash) {
                Some(metadata) => metadata.otrsi(),
                None => None,
            },
        }
    }

    pub fn ytrsi(&self, hash: &MessageId) -> Option<MilestoneIndex> {
        match self.solid_entry_points.get(hash) {
            Some(sep) => Some(*sep.value()),
            None => match self.get_metadata(hash) {
                Some(metadata) => metadata.ytrsi(),
                None => None,
            },
        }
    }

    pub async fn insert_tip(&self, message_id: MessageId, parent1: MessageId, parent2: MessageId) {
        self.tip_pool
            .lock()
            .await
            .insert(&self, message_id, parent1, parent2)
            .await;
    }

    pub async fn update_tip_scores(&self) {
        self.tip_pool.lock().await.update_scores(&self).await;
    }

    pub async fn get_messages_to_approve(&self) -> Option<(MessageId, MessageId)> {
        self.tip_pool.lock().await.two_non_lazy_tips()
    }

    pub async fn reduce_tips(&self) {
        self.tip_pool.lock().await.reduce_tips();
    }

    pub(crate) async fn non_lazy_tips_num(&self) -> usize {
        self.tip_pool.lock().await.non_lazy_tips().len()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{tangle::MessageMetadata, MilestoneIndex};
//
//     use bee_tangle::traversal;
//     use bee_test::{field::rand_trits_field, message::create_random_attached_tx};
//
//     #[test]
//     fn confirm_message() {
//         // Example from https://github.com/iotaledger/protocol-rfcs/blob/master/text/0005-white-flag/0005-white-flag.md
//
//         let tangle = MsTangle::new();
//
//         // Creates solid entry points
//         let sep1 = rand_trits_field::<MessageId>();
//         let sep2 = rand_trits_field::<MessageId>();
//         let sep3 = rand_trits_field::<MessageId>();
//         let sep4 = rand_trits_field::<MessageId>();
//         let sep5 = rand_trits_field::<MessageId>();
//         let sep6 = rand_trits_field::<MessageId>();
//
//         // Adds solid entry points
//         tangle.add_solid_entry_point(sep1, MilestoneIndex(0));
//         tangle.add_solid_entry_point(sep2, MilestoneIndex(1));
//         tangle.add_solid_entry_point(sep3, MilestoneIndex(2));
//         tangle.add_solid_entry_point(sep4, MilestoneIndex(3));
//         tangle.add_solid_entry_point(sep5, MilestoneIndex(4));
//         tangle.add_solid_entry_point(sep6, MilestoneIndex(5));
//
//         // Links messages
//         let (a_hash, a) = create_random_attached_tx(sep1, sep2);
//         let (b_hash, b) = create_random_attached_tx(sep3, sep4);
//         let (c_hash, c) = create_random_attached_tx(sep5, sep6);
//         let (d_hash, d) = create_random_attached_tx(b_hash, a_hash);
//         let (e_hash, e) = create_random_attached_tx(b_hash, a_hash);
//         let (f_hash, f) = create_random_attached_tx(c_hash, b_hash);
//         let (g_hash, g) = create_random_attached_tx(e_hash, d_hash);
//         let (h_hash, h) = create_random_attached_tx(f_hash, e_hash);
//         let (i_hash, i) = create_random_attached_tx(c_hash, f_hash);
//         let (j_hash, j) = create_random_attached_tx(h_hash, g_hash);
//         let (k_hash, k) = create_random_attached_tx(i_hash, h_hash);
//         let (l_hash, l) = create_random_attached_tx(j_hash, g_hash);
//         let (m_hash, m) = create_random_attached_tx(h_hash, j_hash);
//         let (n_hash, n) = create_random_attached_tx(k_hash, h_hash);
//         let (o_hash, o) = create_random_attached_tx(i_hash, k_hash);
//         let (p_hash, p) = create_random_attached_tx(i_hash, k_hash);
//         let (q_hash, q) = create_random_attached_tx(m_hash, l_hash);
//         let (r_hash, r) = create_random_attached_tx(m_hash, l_hash);
//         let (s_hash, s) = create_random_attached_tx(o_hash, n_hash);
//         let (t_hash, t) = create_random_attached_tx(p_hash, o_hash);
//         let (u_hash, u) = create_random_attached_tx(r_hash, q_hash);
//         let (v_hash, v) = create_random_attached_tx(s_hash, r_hash);
//         let (w_hash, w) = create_random_attached_tx(t_hash, s_hash);
//         let (x_hash, x) = create_random_attached_tx(u_hash, q_hash);
//         let (y_hash, y) = create_random_attached_tx(v_hash, u_hash);
//         let (z_hash, z) = create_random_attached_tx(s_hash, v_hash);
//
//         // Confirms messages
//         // TODO uncomment when confirmation index
//         // tangle.confirm_message(a_hash, 1);
//         // tangle.confirm_message(b_hash, 1);
//         // tangle.confirm_message(c_hash, 1);
//         // tangle.confirm_message(d_hash, 2);
//         // tangle.confirm_message(e_hash, 1);
//         // tangle.confirm_message(f_hash, 1);
//         // tangle.confirm_message(g_hash, 2);
//         // tangle.confirm_message(h_hash, 1);
//         // tangle.confirm_message(i_hash, 2);
//         // tangle.confirm_message(j_hash, 2);
//         // tangle.confirm_message(k_hash, 2);
//         // tangle.confirm_message(l_hash, 2);
//         // tangle.confirm_message(m_hash, 2);
//         // tangle.confirm_message(n_hash, 2);
//         // tangle.confirm_message(o_hash, 2);
//         // tangle.confirm_message(p_hash, 3);
//         // tangle.confirm_message(q_hash, 3);
//         // tangle.confirm_message(r_hash, 2);
//         // tangle.confirm_message(s_hash, 2);
//         // tangle.confirm_message(t_hash, 3);
//         // tangle.confirm_message(u_hash, 3);
//         // tangle.confirm_message(v_hash, 2);
//         // tangle.confirm_message(w_hash, 3);
//         // tangle.confirm_message(x_hash, 3);
//         // tangle.confirm_message(y_hash, 3);
//         // tangle.confirm_message(z_hash, 3);
//
//         // Constructs the graph
//         tangle.insert(a, a_hash, MessageMetadata::new());
//         tangle.insert(b, b_hash, MessageMetadata::new());
//         tangle.insert(c, c_hash, MessageMetadata::new());
//         tangle.insert(d, d_hash, MessageMetadata::new());
//         tangle.insert(e, e_hash, MessageMetadata::new());
//         tangle.insert(f, f_hash, MessageMetadata::new());
//         tangle.insert(g, g_hash, MessageMetadata::new());
//         tangle.insert(h, h_hash, MessageMetadata::new());
//         tangle.insert(i, i_hash, MessageMetadata::new());
//         tangle.insert(j, j_hash, MessageMetadata::new());
//         tangle.insert(k, k_hash, MessageMetadata::new());
//         tangle.insert(l, l_hash, MessageMetadata::new());
//         tangle.insert(m, m_hash, MessageMetadata::new());
//         tangle.insert(n, n_hash, MessageMetadata::new());
//         tangle.insert(o, o_hash, MessageMetadata::new());
//         tangle.insert(p, p_hash, MessageMetadata::new());
//         tangle.insert(q, q_hash, MessageMetadata::new());
//         tangle.insert(r, r_hash, MessageMetadata::new());
//         tangle.insert(s, s_hash, MessageMetadata::new());
//         tangle.insert(t, t_hash, MessageMetadata::new());
//         tangle.insert(u, u_hash, MessageMetadata::new());
//         tangle.insert(v, v_hash, MessageMetadata::new());
//         tangle.insert(w, w_hash, MessageMetadata::new());
//         tangle.insert(x, x_hash, MessageMetadata::new());
//         tangle.insert(y, y_hash, MessageMetadata::new());
//         tangle.insert(z, z_hash, MessageMetadata::new());
//
//         let mut hashes = Vec::new();
//
//         traversal::visit_children_depth_first(
//             &tangle.inner,
//             v_hash,
//             |_, _| true,
//             |hash, _tx, _metadata| hashes.push(*hash),
//             |_| (),
//         );
//
//         // TODO Remove when we have confirmation index
//         assert_eq!(hashes.len(), 18);
//
//         assert_eq!(hashes[0], a_hash);
//         assert_eq!(hashes[1], b_hash);
//         assert_eq!(hashes[2], d_hash);
//         assert_eq!(hashes[3], e_hash);
//         assert_eq!(hashes[4], g_hash);
//         assert_eq!(hashes[5], c_hash);
//         assert_eq!(hashes[6], f_hash);
//         assert_eq!(hashes[7], h_hash);
//         assert_eq!(hashes[8], j_hash);
//         assert_eq!(hashes[9], l_hash);
//         assert_eq!(hashes[10], m_hash);
//         assert_eq!(hashes[11], r_hash);
//         assert_eq!(hashes[12], i_hash);
//         assert_eq!(hashes[13], k_hash);
//         assert_eq!(hashes[14], n_hash);
//         assert_eq!(hashes[15], o_hash);
//         assert_eq!(hashes[16], s_hash);
//         assert_eq!(hashes[17], v_hash);
//
//         // TODO uncomment when we have confirmation index
//         // assert_eq!(hashes.len(), 12);
//         // assert_eq!(hashes[0], d_hash);
//         // assert_eq!(hashes[1], g_hash);
//         // assert_eq!(hashes[2], j_hash);
//         // assert_eq!(hashes[3], l_hash);
//         // assert_eq!(hashes[4], m_hash);
//         // assert_eq!(hashes[5], r_hash);
//         // assert_eq!(hashes[6], i_hash);
//         // assert_eq!(hashes[7], k_hash);
//         // assert_eq!(hashes[8], n_hash);
//         // assert_eq!(hashes[9], o_hash);
//         // assert_eq!(hashes[10], s_hash);
//         // assert_eq!(hashes[11], v_hash);
//     }
// }

// use crate::{
//     milestone::MilestoneIndex,
//     vertex::{MessageRef, Vertex},
// };

// use bee_bundle::{MessageId, Message};

// use std::{
//     collections::MessageIdSet,
//     sync::atomic::{AtomicU32, Ordering},
// };

// use async_std::{
//     sync::{Arc, Barrier},
//     task::block_on,
// };

// use dashmap::{mapref::entry::Entry, DashMap, DashSet};

// use flume::Sender;

// /// A datastructure based on a directed acyclic graph (DAG).
// pub struct Tangle<T> {
//     /// A map between each vertex and the hash of the message the respective vertex represents.
//     pub(crate) vertices: DashMap<MessageId, Vertex<T>>,

//     /// A map between the hash of a message and the hashes of its approvers.
//     pub(crate) approvers: DashMap<MessageId, Vec<MessageId>>,

//     /// A map between the milestone index and hash of the milestone message.
//     milestones: DashMap<MilestoneIndex, MessageId>,

//     /// A set of hashes representing messages deemed solid entry points.
//     solid_entry_points: DashSet<MessageId>,

//     /// The sender side of a channel between the Tangle and the (gossip) solidifier.
//     solidifier_send: Sender<Option<MessageId>>,

//     solid_milestone_index: AtomicU32,
//     snapshot_index: AtomicU32,
//     latest_milestone_index: AtomicU32,

//     drop_barrier: Arc<Barrier>,
// }

// impl<T> Tangle<T> {
//     /// Creates a new `Tangle`.
//     pub(crate) fn new(solidifier_send: Sender<Option<MessageId>>, drop_barrier: Arc<Barrier>) -> Self {
//         Self {
//             vertices: DashMap::new(),
//             approvers: DashMap::new(),
//             solidifier_send,
//             solid_entry_points: DashSet::new(),
//             milestones: DashMap::new(),
//             solid_milestone_index: AtomicU32::new(0),
//             snapshot_index: AtomicU32::new(0),
//             latest_milestone_index: AtomicU32::new(0),
//             drop_barrier,
//         }
//     }

//     /// Inserts a message.
//     ///
//     /// Note: The method assumes that `hash` -> `message` is injective, otherwise unexpected behavior could
//     /// occur.
//     pub async fn insert_message(
//         &'static self,
//         message: Message,
//         hash: MessageId,
//         meta: T,
//     ) -> Option<MessageRef> {
//         match self.approvers.entry(*message.parent1()) {
//             Entry::Occupied(mut entry) => {
//                 let values = entry.get_mut();
//                 values.push(hash);
//             }
//             Entry::Vacant(entry) => {
//                 entry.insert(vec![hash]);
//             }
//         }

//         if message.parent1() != message.parent2() {
//             match self.approvers.entry(*message.parent2()) {
//                 Entry::Occupied(mut entry) => {
//                     let values = entry.get_mut();
//                     values.push(hash);
//                 }
//                 Entry::Vacant(entry) => {
//                     entry.insert(vec![hash]);
//                 }
//             }
//         }

//         let vertex = Vertex::from(message, hash, meta);

//         let tx_ref = vertex.get_ref_to_inner();

//         // TODO: not sure if we want replacement of vertices
//         if self.vertices.insert(hash, vertex).is_none() {
//             match self.solidifier_send.send(Some(hash)) {
//                 Ok(()) => (),
//                 Err(e) => todo!("log warning"),
//             }

//             Some(tx_ref)
//         } else {
//             None
//         }
//     }

//     pub(crate) fn shutdown(&self) {
//         // `None` will cause the worker to finish
//         self.solidifier_send.send(None).expect("error sending shutdown signal");
//         block_on(self.drop_barrier.wait());
//     }

//     /// Returns a reference to a message, if it's available in the local Tangle.
//     pub fn get_message(&'static self, hash: &MessageId) -> Option<MessageRef> {
//         self.vertices.get(hash).map(|v| v.get_ref_to_inner())
//     }

//     /// Returns whether the message is stored in the Tangle.
//     pub fn contains_message(&'static self, hash: &MessageId) -> bool {
//         self.vertices.contains_key(hash)
//     }

//     /// Returns whether the message associated with `hash` is solid.
//     ///
//     /// Note: This function is _eventually consistent_ - if `true` is returned, solidification has
//     /// definitely occurred. If `false` is returned, then solidification has probably not occurred,
//     /// or solidification information has not yet been fully propagated.
//     pub fn is_solid_message(&'static self, hash: &MessageId) -> bool {
//         if self.is_solid_entry_point(hash) {
//             true
//         } else {
//             self.vertices.get(hash).map(|r| r.value().is_solid()).unwrap_or(false)
//         }
//     }

//     /// Adds the `hash` of a milestone identified by its milestone `index`.
//     pub fn add_milestone(&'static self, index: MilestoneIndex, hash: MessageId) {
//         self.milestones.insert(index, hash);
//         if let Some(mut vertex) = self.vertices.get_mut(&hash) {
//             vertex.set_milestone();
//         }
//     }

//     /// Removes the hash of a milestone.
//     pub fn remove_milestone(&'static self, index: MilestoneIndex) {
//         self.milestones.remove(&index);
//     }

//     /// Returns the milestone message corresponding to the given milestone `index`.
//     pub fn get_milestone(&'static self, index: MilestoneIndex) -> Option<MessageRef> {
//         match self.get_milestone_message_id(index) {
//             None => None,
//             Some(hash) => self.get_message(&hash),
//         }
//     }

//     /// Returns a [`VertexRef`] linked to the specified milestone, if it's available in the local Tangle.
//     pub fn get_latest_milestone(&'static self) -> Option<MessageRef> {
//         todo!("get the latest milestone index, get the message hash from it, and query the Tangle for it")
//     }

//     /// Returns the hash of a milestone.
//     pub fn get_milestone_message_id(&'static self, index: MilestoneIndex) -> Option<MessageId> {
//         match self.milestones.get(&index) {
//             None => None,
//             Some(v) => Some(*v),
//         }
//     }

//     /// Returns whether the milestone index maps to a know milestone hash.
//     pub fn contains_milestone(&'static self, index: MilestoneIndex) -> bool {
//         self.milestones.contains_key(&index)
//     }

//     /// Retreives the solid milestone index.
//     pub fn get_solid_milestone_index(&'static self) -> MilestoneIndex {
//         self.solid_milestone_index.load(Ordering::Relaxed).into()
//     }

//     /// Updates the solid milestone index to `new_index`.
//     pub fn update_solid_milestone_index(&'static self, new_index: MilestoneIndex) {
//         self.solid_milestone_index.store(*new_index, Ordering::Relaxed);
//     }

//     /// Retreives the snapshot milestone index.
//     pub fn get_snapshot_index(&'static self) -> MilestoneIndex {
//         self.snapshot_index.load(Ordering::Relaxed).into()
//     }

//     /// Updates the snapshot milestone index to `new_index`.
//     pub fn update_snapshot_index(&'static self, new_index: MilestoneIndex) {
//         self.snapshot_index.store(*new_index, Ordering::Relaxed);
//     }

//     /// Retreives the latest milestone index.
//     pub fn get_latest_milestone_index(&'static self) -> MilestoneIndex {
//         self.latest_milestone_index.load(Ordering::Relaxed).into()
//     }

//     /// Updates the latest milestone index to `new_index`.
//     pub fn update_latest_milestone_index(&'static self, new_index: MilestoneIndex) {
//         self.latest_milestone_index.store(*new_index, Ordering::Relaxed);
//     }

//     /// Adds `hash` to the set of solid entry points.
//     pub fn add_solid_entry_point(&'static self, hash: MessageId) {
//         self.solid_entry_points.insert(hash);
//     }

//     /// Removes `hash` from the set of solid entry points.
//     pub fn remove_solid_entry_point(&'static self, hash: MessageId) {
//         self.solid_entry_points.remove(&hash);
//     }

//     /// Returns whether the message associated `hash` is a solid entry point.
//     pub fn is_solid_entry_point(&'static self, hash: &MessageId) -> bool {
//         self.solid_entry_points.contains(hash)
//     }

//     /// Checks if the tangle is synced or not
//     pub fn is_synced(&'static self) -> bool {
//         self.get_solid_milestone_index() == self.get_latest_milestone_index()
//     }

//     /// Returns the current size of the Tangle.
//     pub fn size(&'static self) -> usize {
//         self.vertices.len()
//     }

//     /// Starts a walk beginning at a `start` vertex identified by its associated message hash
//     /// traversing its children/approvers for as long as those satisfy a given `filter`.
//     ///
//     /// Returns a list of descendents of `start`. It is ensured, that all elements of that list
//     /// are connected through the parent1.
//     pub fn parent1_walk_approvers<F>(&'static self, start: MessageId, filter: F) -> Vec<(MessageRef, MessageId)>
//     where
//         F: Fn(&MessageRef) -> bool,
//     {
//         let mut approvees = vec![];
//         let mut collected = vec![];

//         if let Some(approvee_ref) = self.vertices.get(&start) {
//             let approvee_vtx = approvee_ref.value();
//             let approvee = approvee_vtx.get_ref_to_inner();

//             if filter(&approvee) {
//                 approvees.push(start);
//                 collected.push((approvee, approvee_vtx.get_id()));

//                 while let Some(approvee_hash) = approvees.pop() {
//                     if let Some(approvers_ref) = self.approvers.get(&approvee_hash) {
//                         for approver_hash in approvers_ref.value() {
//                             if let Some(approver_ref) = self.vertices.get(approver_hash) {
//                                 let approver = approver_ref.value().get_ref_to_inner();

//                                 if *approver.parent1() == approvee_hash && filter(&approver) {
//                                     approvees.push(*approver_hash);
//                                     collected.push((approver, approver_ref.value().get_id()));
//                                     // NOTE: For simplicity reasons we break here, and assume, that there can't be
//                                     // a second approver that passes the filter
//                                     break;
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }

//         collected
//     }

//     /// Starts a walk beginning at a `start` vertex identified by its associated message hash
//     /// traversing its ancestors/approvees for as long as those satisfy a given `filter`.
//     ///
//     /// Returns a list of ancestors of `start`. It is ensured, that all elements of that list
//     /// are connected through the parent1.
//     pub fn parent1_walk_approvees<F>(&'static self, start: MessageId, filter: F) -> Vec<(MessageRef, MessageId)>
//     where
//         F: Fn(&MessageRef) -> bool,
//     {
//         let mut approvers = vec![start];
//         let mut collected = vec![];

//         while let Some(approver_hash) = approvers.pop() {
//             if let Some(approver_ref) = self.vertices.get(&approver_hash) {
//                 let approver_vtx = approver_ref.value();
//                 let approver = approver_vtx.get_ref_to_inner();

//                 if !filter(&approver) {
//                     break;
//                 } else {
//                     approvers.push(approver.parent1().clone());
//                     collected.push((approver, approver_vtx.get_id()));
//                 }
//             }
//         }

//         collected
//     }

//     /// Walks all approvers given a starting hash `root`.
//     pub fn walk_approvees_depth_first<Mapping, Follow, Missing>(
//         &'static self,
//         root: MessageId,
//         mut map: Mapping,
//         should_follow: Follow,
//         mut on_missing: Missing,
//     ) where
//         Mapping: FnMut(&MessageRef),
//         Follow: Fn(&Vertex<T>) -> bool,
//         Missing: FnMut(&MessageId),
//     {
//         let mut non_analyzed_hashes = Vec::new();
//         let mut analyzed_hashes = MessageIdSet::new();

//         non_analyzed_hashes.push(root);

//         while let Some(hash) = non_analyzed_hashes.pop() {
//             if !analyzed_hashes.contains(&hash) {
//                 match self.vertices.get(&hash) {
//                     Some(vertex) => {
//                         let vertex = vertex.value();
//                         let message = vertex.get_ref_to_inner();

//                         map(&message);

//                         if should_follow(vertex) {
//                             non_analyzed_hashes.push(*message.parent2());
//                             non_analyzed_hashes.push(*message.parent1());
//                         }
//                     }
//                     None => {
//                         if !self.is_solid_entry_point(&hash) {
//                             on_missing(&hash);
//                         }
//                     }
//                 }
//                 analyzed_hashes.insert(hash);
//             }
//         }
//     }

//     /// Walks all approvers in a post order DFS way through parent1 then parent2.
//     pub fn walk_approvers_post_order_dfs<Mapping, Follow, Missing>(
//         &'static self,
//         root: MessageId,
//         mut map: Mapping,
//         should_follow: Follow,
//         mut on_missing: Missing,
//     ) where
//         Mapping: FnMut(&MessageId, &MessageRef),
//         Follow: Fn(&Vertex<T>) -> bool,
//         Missing: FnMut(&MessageId),
//     {
//         let mut non_analyzed_hashes = Vec::new();
//         let mut analyzed_hashes = MessageIdSet::new();

//         non_analyzed_hashes.push(root);

//         while let Some(hash) = non_analyzed_hashes.last() {
//             match self.vertices.get(hash) {
//                 Some(vertex) => {
//                     let vertex = vertex.value();
//                     let message = vertex.get_ref_to_inner();

//                     // TODO add follow
//                     if analyzed_hashes.contains(message.parent1()) &&
// analyzed_hashes.contains(message.parent2()) {                         map(hash, &message);
//                         analyzed_hashes.insert(hash.clone());
//                         non_analyzed_hashes.pop();
//                     // TODO add follow
//                     } else if !analyzed_hashes.contains(message.parent1()) {
//                         non_analyzed_hashes.push(*message.parent1());
//                     // TODO add follow
//                     } else if !analyzed_hashes.contains(message.parent2()) {
//                         non_analyzed_hashes.push(*message.parent2());
//                     }
//                 }
//                 None => {
//                     if !self.is_solid_entry_point(hash) {
//                         on_missing(hash);
//                     }
//                     analyzed_hashes.insert(hash.clone());
//                     non_analyzed_hashes.pop();
//                 }
//             }
//         }
//     }

//     #[cfg(test)]
//     fn num_approvers(&'static self, hash: &MessageId) -> usize {
//         self.approvers.get(hash).map_or(0, |r| r.value().len())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::*;

//     #[test]
//     #[serial]
//     fn update_and_get_snapshot_index() {
//         init();
//         let tangle = tangle();

//         tangle.update_snapshot_index(1368160.into());

//         assert_eq!(1368160, *tangle.get_snapshot_index());
//         drop();
//     }

//     #[test]
//     #[serial]
//     fn update_and_get_solid_milestone_index() {
//         init();
//         let tangle = tangle();

//         tangle.update_solid_milestone_index(1368167.into());

//         assert_eq!(1368167, *tangle.get_solid_milestone_index());
//         drop();
//     }

//     #[test]
//     #[serial]
//     fn update_and_get_latest_milestone_index() {
//         init();
//         let tangle = tangle();

//         tangle.update_latest_milestone_index(1368168.into());

//         assert_eq!(1368168, *tangle.get_latest_milestone_index());
//         drop();
//     }

// ----

// pub use milestone::MilestoneIndex;
// pub use tangle::Tangle;
// pub use vertex::MessageRef;

// //mod milestone;
// //mod solidifier;
// mod tangle;
// mod vertex;

// use solidifier::SolidifierState;

// use async_std::{
//     sync::{channel, Arc, Barrier},
//     task::spawn,
// };

// use bee_bundle::MessageId;

// use std::{
//     ptr,
//     sync::atomic::{AtomicBool, AtomicPtr, Ordering},
// };

// static TANGLE: AtomicPtr<Tangle<u8>> = AtomicPtr::new(ptr::null_mut());
// static INITIALIZED: AtomicBool = AtomicBool::new(false);

// const SOLIDIFIER_CHAN_CAPACITY: usize = 1000;

// /// Initializes the Tangle singleton.
// pub fn init() {
//     if !INITIALIZED.compare_and_swap(false, true, Ordering::Relaxed) {
//         let (sender, receiver) = flume::bounded::<Option<MessageId>>(SOLIDIFIER_CHAN_CAPACITY);

//         let drop_barrier = async_std::sync::Arc::new(Barrier::new(2));

//         TANGLE.store(
//             Box::into_raw(Tangle::new(sender, drop_barrier.clone()).into()),
//             Ordering::Relaxed,
//         );

//         spawn(SolidifierState::new(receiver, drop_barrier).run());
//     } else {
//         drop();
//         panic!("Already initialized");
//     }
// }

// /// Returns the singleton instance of the Tangle.
// pub fn tangle() -> &'static Tangle<u8> {
//     let tangle = TANGLE.load(Ordering::Relaxed);
//     if tangle.is_null() {
//         panic!("Tangle cannot be null");
//     } else {
//         unsafe { &*tangle }
//     }
// }

// /// Drops the Tangle singleton.
// pub fn drop() {
//     if INITIALIZED.compare_and_swap(true, false, Ordering::Relaxed) {
//         tangle().shutdown();

//         let tangle = TANGLE.swap(ptr::null_mut(), Ordering::Relaxed);
//         if !tangle.is_null() {
//             let _ = unsafe { Box::from_raw(tangle) };
//         }
//     } else {
//         panic!("Already dropped");
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use serial_test::serial;

//     #[test]
//     #[serial]
//     fn init_get_and_drop() {
//         init();
//         let _ = tangle();
//         drop();
//     }

//     #[test]
//     #[should_panic]
//     #[serial]
//     fn double_init_should_panic() {
//         init();
//         init();
//     }

//     #[test]
//     #[should_panic]
//     #[serial]
//     fn double_drop_should_panic() {
//         init();
//         drop();
//         drop();
//     }

//     #[test]
//     #[should_panic]
//     #[serial]
//     fn drop_without_init_should_panic() {
//         drop();
//     }

//     #[test]
//     #[should_panic]
//     #[serial]
//     fn get_without_init_should_panic() {
//         let _ = tangle();
//         drop();
//     }
// }
