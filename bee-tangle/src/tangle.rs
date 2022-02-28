// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::TangleConfig,
    metadata::{IndexId, MessageMetadata},
    solid_entry_point::SolidEntryPoint,
    storage::StorageBackend,
    urts::UrtsTipPool,
    vertex::Vertex,
    vertices::Vertices,
    MessageRef,
};

use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    Message, MessageId,
};
use bee_runtime::resource::ResourceHandle;

use hashbrown::HashMap;
use log::info;
use parking_lot::Mutex;
use ref_cast::RefCast;

use std::{
    marker::PhantomData,
    ops::Deref,
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
};

const DEFAULT_CACHE_LEN: usize = 100_000;
const CACHE_THRESHOLD_FACTOR: f64 = 0.1;
const SYNCED_THRESHOLD: u32 = 2;
const CONFIRMED_THRESHOLD: u32 = 2;

/// A Tangle wrapper designed to encapsulate milestone state.
pub struct Tangle<B> {
    config: TangleConfig,
    vertices: Vertices,
    max_len: AtomicUsize,
    storage: ResourceHandle<B>,
    milestones: Mutex<HashMap<MilestoneIndex, Milestone>>,
    solid_entry_points: Mutex<HashMap<SolidEntryPoint, MilestoneIndex>>,
    latest_milestone_index: AtomicU32,
    solid_milestone_index: AtomicU32,
    confirmed_milestone_index: AtomicU32,
    snapshot_index: AtomicU32,
    pruning_index: AtomicU32,
    entry_point_index: AtomicU32,
    tip_pool: Mutex<UrtsTipPool>,
}

impl<B: StorageBackend> Tangle<B> {
    /// Create a new `Tangle` instance with the given configuration and storage handle.
    pub fn new(config: TangleConfig, storage: ResourceHandle<B>) -> Self {
        Self {
            vertices: Vertices::new(config.num_partitions()),
            max_len: AtomicUsize::new(DEFAULT_CACHE_LEN),
            storage,
            milestones: Default::default(),
            solid_entry_points: Default::default(),
            latest_milestone_index: Default::default(),
            solid_milestone_index: Default::default(),
            confirmed_milestone_index: Default::default(),
            snapshot_index: Default::default(),
            pruning_index: Default::default(),
            entry_point_index: Default::default(),
            tip_pool: Mutex::new(UrtsTipPool::new(&config)),
            config,
        }
    }

    /// Shut down the tangle, terminating any and all worker tasks.
    pub fn shutdown(self) {
        // TODO: Write back changes by calling self.inner.shutdown()
    }

    /// Get the configuration of this tangle.
    pub fn config(&self) -> &TangleConfig {
        &self.config
    }

    /// Insert a message into the tangle.
    pub fn insert(&self, message: Message, message_id: MessageId, metadata: MessageMetadata) -> Option<MessageRef> {
        let exists = self.pull_message(&message_id, true);

        let msg = self.insert_inner(message_id, message.clone(), metadata, !exists);

        self.vertices
            .get_mut_map(&message_id, |v| v.allow_eviction())
            .expect("Just-inserted message is missing");

        if msg.is_some() {
            // Write parents to DB
            for &parent in message.parents().iter() {
                self.storage
                    .insert(&(parent, message_id), &())
                    .unwrap_or_else(|e| info!("Failed to update approvers for message {:?}", e));
            }

            // Insert into backend using hooks
            self.storage_insert(message_id, message, metadata)
                .unwrap_or_else(|e| info!("Failed to insert message {:?}", e));
        }

        msg
    }

    /// Add a milestone to the tangle.
    pub fn add_milestone(&self, idx: MilestoneIndex, milestone: Milestone) {
        // TODO: only insert if vacant
        self.update_metadata(milestone.message_id(), |metadata| {
            metadata.flags_mut().set_milestone(true);
            metadata.set_milestone_index(idx);
            metadata.set_omrsi(IndexId::new(idx, *milestone.message_id()));
            metadata.set_ymrsi(IndexId::new(idx, *milestone.message_id()));
        });
        self.storage()
            .insert(&idx, &milestone)
            .unwrap_or_else(|e| info!("Failed to insert message {:?}", e));
        self.milestones.lock().insert(idx, milestone);
    }

    /// Remove a milestone from the tangle.
    pub fn remove_milestone(&self, index: MilestoneIndex) {
        self.milestones.lock().remove(&index);
    }

    fn pull_milestone(&self, idx: MilestoneIndex) -> Option<MessageId> {
        if let Some(milestone) = self.storage().fetch(&idx).unwrap_or_else(|e| {
            info!("Failed to insert message {:?}", e);
            None
        }) {
            let message_id = *self.milestones.lock().entry(idx).or_insert(milestone).message_id();

            Some(message_id)
        } else {
            None
        }
    }

    /// Get the milestone from the tangle that corresponds to the given milestone index.
    pub fn get_milestone(&self, index: MilestoneIndex) -> Option<Milestone> {
        self.milestones.lock().get(&index).cloned()
    }

    /// Get the message associated with the given milestone index from the tangle.
    pub fn get_milestone_message(&self, index: MilestoneIndex) -> Option<MessageRef> {
        // TODO: use combinator instead of match
        match self.get_milestone_message_id(index) {
            None => None,
            Some(ref hash) => self.get(hash),
        }
    }

    /// Get the message ID associated with the given milestone index from the tangle.
    pub fn get_milestone_message_id(&self, index: MilestoneIndex) -> Option<MessageId> {
        let message_id = self.milestones.lock().get(&index).map(|m| *m.message_id());

        // TODO: use combinator instead of match
        match message_id {
            Some(m) => Some(m),
            None => Some(self.pull_milestone(index)?),
        }
    }

    /// Return whether the tangle contains the given milestone index.
    pub fn contains_milestone(&self, idx: MilestoneIndex) -> bool {
        // Not using `||` as its first operand would keep the lock alive causing a deadlock with its second operand.
        if self.milestones.lock().contains_key(&idx) {
            return true;
        }
        self.pull_milestone(idx).is_some()
    }

    /// Get the index of the latest milestone.
    pub fn get_latest_milestone_index(&self) -> MilestoneIndex {
        self.latest_milestone_index.load(Ordering::Relaxed).into()
    }

    /// Update the index of the lastest milestone.
    pub fn update_latest_milestone_index(&self, new_index: MilestoneIndex) {
        // TODO: `fetch_max`? Swap and ensure the old is smaller?
        self.latest_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    /// Get the latest solid milestone index.
    pub fn get_solid_milestone_index(&self) -> MilestoneIndex {
        self.solid_milestone_index.load(Ordering::Relaxed).into()
    }

    /// Update the latest solid milestone index.
    pub fn update_solid_milestone_index(&self, new_index: MilestoneIndex) {
        self.solid_milestone_index.store(*new_index, Ordering::Relaxed);

        // TODO: Formalise this a little better
        let new_len = ((1000.0 + self.get_sync_threshold() as f32 * 500.0) as usize)
            .min(DEFAULT_CACHE_LEN)
            .max(8192);
        self.resize(new_len);
    }

    /// Get the latest confirmed milestone index.
    pub fn get_confirmed_milestone_index(&self) -> MilestoneIndex {
        self.confirmed_milestone_index.load(Ordering::Relaxed).into()
    }

    /// Update the latest confirmed milestone index.
    pub fn update_confirmed_milestone_index(&self, new_index: MilestoneIndex) {
        self.confirmed_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    /// Get the snapshot index.
    pub fn get_snapshot_index(&self) -> MilestoneIndex {
        self.snapshot_index.load(Ordering::Relaxed).into()
    }

    /// Update the snapshot index.
    pub fn update_snapshot_index(&self, new_index: MilestoneIndex) {
        self.snapshot_index.store(*new_index, Ordering::Relaxed);
    }

    /// Get the pruning index.
    pub fn get_pruning_index(&self) -> MilestoneIndex {
        self.pruning_index.load(Ordering::Relaxed).into()
    }

    /// Update the pruning index.
    pub fn update_pruning_index(&self, new_index: MilestoneIndex) {
        self.pruning_index.store(*new_index, Ordering::Relaxed);
    }

    /// Get the entry point index.
    pub fn get_entry_point_index(&self) -> MilestoneIndex {
        self.entry_point_index.load(Ordering::Relaxed).into()
    }

    /// Update the entry point index.
    pub fn update_entry_point_index(&self, new_index: MilestoneIndex) {
        self.entry_point_index.store(*new_index, Ordering::Relaxed);
    }

    /// Return whether the tangle is within the default sync threshold.
    pub fn is_synced(&self) -> bool {
        // TODO reduce to one atomic value ?
        self.is_synced_threshold(SYNCED_THRESHOLD)
    }

    /// Get the number of milestones until the tangle is synced.
    pub fn get_sync_threshold(&self) -> u32 {
        // TODO reduce to one atomic value ?
        self.get_latest_milestone_index()
            .saturating_sub(*self.get_solid_milestone_index())
    }

    /// Return whether the tangle is within the given sync threshold.
    pub fn is_synced_threshold(&self, threshold: u32) -> bool {
        // TODO reduce to one atomic value ?
        *self.get_solid_milestone_index() >= self.get_latest_milestone_index().saturating_sub(threshold)
    }

    /// Return whether the tangle is fully confirmed.
    pub fn is_confirmed(&self) -> bool {
        // TODO reduce to one atomic value ?
        self.is_confirmed_threshold(CONFIRMED_THRESHOLD)
    }

    /// Return whether the tangle is within the given confirmation threshold.
    pub fn is_confirmed_threshold(&self, threshold: u32) -> bool {
        // TODO reduce to one atomic value ?
        *self.get_confirmed_milestone_index() >= self.get_latest_milestone_index().saturating_sub(threshold)
    }

    /// Get the milestone index associated with the given solid entry point.
    pub fn get_solid_entry_point_index(&self, sep: &SolidEntryPoint) -> Option<MilestoneIndex> {
        self.solid_entry_points.lock().get(sep).copied()
    }

    /// Add the given solid entry point to the given milestone index.
    pub fn add_solid_entry_point(&self, sep: SolidEntryPoint, index: MilestoneIndex) {
        self.solid_entry_points.lock().insert(sep, index);
    }

    /// Returns a copy of all solid entry points.
    pub fn get_solid_entry_points(&self) -> HashMap<SolidEntryPoint, MilestoneIndex> {
        self.solid_entry_points.lock().clone()
    }

    /// Removes the given solid entry point from the set of solid entry points.
    pub fn remove_solid_entry_point(&self, sep: &SolidEntryPoint) {
        self.solid_entry_points.lock().remove(sep);
    }

    /// Clear all solid entry points.
    pub fn clear_solid_entry_points(&self) {
        self.solid_entry_points.lock().clear();
    }

    /// Replaces all solid entry points.
    pub fn replace_solid_entry_points(&self, new_seps: impl IntoIterator<Item = (SolidEntryPoint, MilestoneIndex)>) {
        let mut seps = self.solid_entry_points.lock();
        seps.clear();
        seps.extend(new_seps);
    }

    /// Returns whether the message associated with given solid entry point is a solid entry point.
    pub fn is_solid_entry_point(&self, id: &MessageId) -> bool {
        self.solid_entry_points
            .lock()
            .contains_key(SolidEntryPoint::ref_cast(id))
    }

    /// Returns whether the message associated with the given message ID is solid.
    pub fn is_solid_message(&self, id: &MessageId) -> bool {
        if self.is_solid_entry_point(id) {
            true
        } else {
            self.get_metadata(id)
                .map(|metadata| metadata.flags().is_solid())
                .unwrap_or(false)
        }
    }

    /// Get the oldest milestone root snapshot index.
    pub fn omrsi(&self, id: &MessageId) -> Option<IndexId> {
        match self.solid_entry_points.lock().get(SolidEntryPoint::ref_cast(id)) {
            Some(sep) => Some(IndexId::new(*sep, *id)),
            None => match self.get_metadata(id) {
                Some(metadata) => metadata.omrsi(),
                None => None,
            },
        }
    }

    /// Get the youngest milestone root snapshot index.
    pub fn ymrsi(&self, id: &MessageId) -> Option<IndexId> {
        match self.solid_entry_points.lock().get(SolidEntryPoint::ref_cast(id)) {
            Some(sep) => Some(IndexId::new(*sep, *id)),
            None => match self.get_metadata(id) {
                Some(metadata) => metadata.ymrsi(),
                None => None,
            },
        }
    }

    /// Insert the given message ID and parents as a tip.
    pub fn insert_tip(&self, message_id: MessageId, parents: Vec<MessageId>) {
        self.tip_pool.lock().insert(self, message_id, parents);
    }

    /// Update tip scores.
    pub fn update_tip_scores(&self) {
        self.tip_pool.lock().update_scores(self);
    }

    /// Return messages that require approving.
    pub fn get_messages_to_approve(&self) -> Option<Vec<MessageId>> {
        self.tip_pool.lock().choose_non_lazy_tips()
    }

    /// Reduce tips.
    pub fn reduce_tips(&self) {
        self.tip_pool.lock().reduce_tips();
    }

    /// Return the number of non-lazy tips.
    pub fn non_lazy_tips_num(&self) -> usize {
        self.tip_pool.lock().non_lazy_tips().len()
    }

    /// Change the maximum number of entries to store in the cache.
    fn resize(&self, len: usize) {
        self.max_len.store(len, Ordering::Relaxed);
    }

    /// Return a reference to the storage  used by this tangle.
    fn storage(&self) -> &B {
        &self.storage
    }

    fn insert_inner(
        &self,
        message_id: MessageId,
        message: Message,
        metadata: MessageMetadata,
        prevent_eviction: bool,
    ) -> Option<MessageRef> {
        let opt = self.vertices.get_mut_or_insert_map(message_id, |vertex| {
            if prevent_eviction {
                vertex.prevent_eviction();
            }
            if vertex.message().is_some() {
                None
            } else {
                let parents = message.parents().clone();

                vertex.insert_message_and_metadata(message, metadata);
                let msg = vertex.message().cloned();

                Some((msg, parents.to_vec()))
            }
        });

        let msg = if let Some((msg, parents)) = opt {
            // Insert children for parents
            for &parent in parents.iter() {
                self.vertices.get_mut_or_insert_map(parent, |v| v.add_child(message_id));
            }
            msg
        } else {
            None
        };

        self.perform_eviction();

        msg
    }

    /// Get the data of a vertex associated with the given `message_id`.
    fn get_mut_and_then<R>(&self, message_id: &MessageId, f: impl FnOnce(&mut Vertex) -> Option<R>) -> Option<R> {
        let exists = self.pull_message(message_id, true);

        self.vertices.get_mut_and_then(message_id, |v| {
            if exists {
                v.allow_eviction();
            }
            f(v)
        })
    }

    /// Get the data of a vertex associated with the given `message_id`.
    pub fn get(&self, message_id: &MessageId) -> Option<MessageRef> {
        self.get_mut_and_then(message_id, |v| v.message().cloned())
    }

    fn contains_inner(&self, message_id: &MessageId) -> bool {
        self.vertices
            .get_map(message_id, |v| v.message().is_some())
            .unwrap_or(false)
    }

    /// Returns whether the message is stored in the Tangle.
    pub fn contains(&self, message_id: &MessageId) -> bool {
        self.contains_inner(message_id) || self.pull_message(message_id, false)
    }

    /// Get the metadata of a vertex associated with the given `message_id`.
    pub fn get_metadata(&self, message_id: &MessageId) -> Option<MessageMetadata> {
        self.get_mut_and_then(message_id, |v| v.metadata().cloned())
    }

    /// Get the metadata of a vertex associated with the given `message_id`.
    pub fn get_vertex_and_then<T>(&self, message_id: &MessageId, f: impl FnOnce(&Vertex) -> Option<T>) -> Option<T> {
        let exists = self.pull_message(message_id, true);

        self.vertices.get_mut_and_then(message_id, |v| {
            if exists {
                v.allow_eviction();
            }
            f(v)
        })
    }

    /// Updates the metadata of a vertex.
    pub fn update_metadata<R, Update>(&self, message_id: &MessageId, update: Update) -> Option<R>
    where
        Update: FnOnce(&mut MessageMetadata) -> R,
    {
        let exists = self.pull_message(message_id, true);

        self.vertices.get_mut_and_then(message_id, |vertex| {
            if exists {
                vertex.allow_eviction();
            }
            let r = vertex.metadata_mut().map(update);

            if let Some((msg, meta)) = vertex.message_and_metadata() {
                let (msg, meta) = ((&**msg).clone(), *meta);

                self.storage_insert(*message_id, msg, meta)
                    .unwrap_or_else(|e| info!("Failed to update metadata for message {:?}", e));
            }

            r
        })
    }

    fn children_inner(&self, message_id: &MessageId) -> Option<impl Deref<Target = Vec<MessageId>> + '_> {
        struct Wrapper<'a> {
            children: Vec<MessageId>,
            phantom: PhantomData<&'a ()>,
        }

        impl<'a> Deref for Wrapper<'a> {
            type Target = Vec<MessageId>;

            fn deref(&self) -> &Self::Target {
                &self.children
            }
        }

        let children_opt = self.vertices.get_and_then(message_id, |v| {
            // Skip approver lists that are not exhaustive
            v.children_exhaustive().then(|| v.children().to_vec())
        });

        let children = match children_opt {
            Some(children) => children,
            None => {
                let to_insert = match self.storage.fetch(message_id) {
                    Err(e) => {
                        info!("Failed to update approvers for message message {:?}", e);
                        Vec::new()
                    }
                    Ok(None) => Vec::new(),
                    Ok(Some(approvers)) => approvers,
                };

                self.vertices.get_mut_or_insert_map(*message_id, |vertex| {
                    vertex.set_exhaustive();

                    for child in to_insert {
                        vertex.add_child(child);
                    }
                    vertex.children().to_vec()
                })
            }
        };

        Some(Wrapper {
            children,
            phantom: PhantomData,
        })
    }

    /// Returns the children of a vertex, if we know about them.
    pub fn get_children(&self, message_id: &MessageId) -> Option<Vec<MessageId>> {
        // Effectively atomic
        self.children_inner(message_id).map(|approvers| approvers.clone())
    }

    // Attempts to pull the message from the storage, returns true if successful.
    fn pull_message(&self, message_id: &MessageId, prevent_eviction: bool) -> bool {
        let contains_now = if prevent_eviction {
            self.vertices
                .get_mut_map(message_id, |v| {
                    if v.message().is_some() {
                        v.prevent_eviction();
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false)
        } else {
            self.contains_inner(message_id)
        };

        // If the tangle already contains the message, do no more work
        if contains_now {
            true
        } else if let Ok(Some((msg, metadata))) = self.storage_get(message_id) {
            self.insert_inner(*message_id, msg, metadata, prevent_eviction);

            true
        } else {
            false
        }
    }

    fn perform_eviction(&self) {
        let max_len = self.max_len.load(Ordering::Relaxed);
        let max_eviction_retries = self.config.max_eviction_retries();

        if self.vertices.len() > max_len {
            while self.vertices.len() > ((1.0 - CACHE_THRESHOLD_FACTOR) * max_len as f64) as usize {
                if self.vertices.pop_random(max_eviction_retries).is_none() {
                    log::warn!(
                        "could not perform cache eviction after {} attempts",
                        max_eviction_retries
                    );

                    break;
                }
            }
        }
    }

    fn storage_get(&self, id: &MessageId) -> Result<Option<(Message, MessageMetadata)>, B::Error> {
        let msg = self.storage.fetch(id)?;
        let meta = self.storage.fetch(id)?;

        Ok(msg.zip(meta))
    }

    fn storage_insert(&self, id: MessageId, tx: Message, metadata: MessageMetadata) -> Result<(), B::Error> {
        self.storage.insert(&id, &tx)?;
        self.storage.insert(&id, &metadata)?;

        Ok(())
    }
}
