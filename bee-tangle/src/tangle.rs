// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::TangleConfig,
    metadata::{IndexId, MessageMetadata},
    solid_entry_point::SolidEntryPoint,
    storage::StorageBackend,
    urts::UrtsTipPool,
    vertex::Vertex,
    MessageRef,
};

use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    Message, MessageId,
};
use bee_runtime::resource::ResourceHandle;

use hashbrown::HashMap;
use log::{info, trace};
use rand::Rng;
use ref_cast::RefCast;
use tokio::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use std::{
    convert::TryInto,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
};

const DEFAULT_CACHE_LEN: usize = 100_000;
const CACHE_THRESHOLD_FACTOR: f64 = 0.1;
const SYNCED_THRESHOLD: u32 = 2;
const CONFIRMED_THRESHOLD: u32 = 2;
const PARTITION_LENGTH: usize = 100;

/// A Tangle wrapper designed to encapsulate milestone state.
pub struct Tangle<B> {
    config: TangleConfig,
    vertices: Vec<RwLock<HashMap<MessageId, Vertex>>>,
    max_len: AtomicUsize,
    len: AtomicUsize,
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
            vertices: (0..PARTITION_LENGTH).map(|_| RwLock::new(HashMap::new())).collect(),
            max_len: AtomicUsize::new(DEFAULT_CACHE_LEN),
            len: AtomicUsize::new(0),
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
    pub async fn shutdown(self) {
        // TODO: Write back changes by calling self.inner.shutdown().await
    }

    /// Get the configuration of this tangle.
    pub fn config(&self) -> &TangleConfig {
        &self.config
    }

    fn hash_intern(&self, message_id: &MessageId) -> usize {
        usize::from_le_bytes(message_id.as_ref()[0..8].try_into().unwrap()) % PARTITION_LENGTH
    }

    fn get_interned_vertices(&self, message_id: &MessageId) -> &RwLock<HashMap<MessageId, Vertex>> {
        &self.vertices[self.hash_intern(message_id)]
    }

    async fn get_interned(&self, message_id: &MessageId) -> Option<impl Deref<Target = Vertex> + '_> {
        RwLockReadGuard::try_map(self.get_interned_vertices(message_id).read().await, |m| {
            m.get(message_id)
        })
        .ok()
    }

    async fn get_interned_mut(&self, message_id: &MessageId) -> Option<impl DerefMut<Target = Vertex> + '_> {
        RwLockWriteGuard::try_map(self.get_interned_vertices(message_id).write().await, |m| {
            m.get_mut(message_id)
        })
        .ok()
    }

    /// Insert a message into the tangle.
    pub async fn insert(
        &self,
        message: Message,
        message_id: MessageId,
        metadata: MessageMetadata,
    ) -> Option<MessageRef> {
        self.pull_message(&message_id).await;

        let msg = self.insert_inner(message_id, message.clone(), metadata).await;

        if msg.is_some() {
            // Write parents to DB
            for &parent in message.parents().iter() {
                self.storage_insert_approver(parent, message_id)
                    .unwrap_or_else(|e| info!("Failed to update approvers for message {:?}", e));
            }

            // Insert into backend using hooks
            self.storage_insert(message_id, message, metadata)
                .unwrap_or_else(|e| info!("Failed to insert message {:?}", e));
        }

        msg
    }

    /// Add a milestone to the tangle.
    pub async fn add_milestone(&self, idx: MilestoneIndex, milestone: Milestone) {
        // TODO: only insert if vacant
        self.update_metadata(milestone.message_id(), |metadata| {
            metadata.flags_mut().set_milestone(true);
            metadata.set_milestone_index(idx);
            metadata.set_omrsi(IndexId::new(idx, *milestone.message_id()));
            metadata.set_ymrsi(IndexId::new(idx, *milestone.message_id()));
        })
        .await;
        self.storage_insert_milestone(idx, &milestone)
            .unwrap_or_else(|e| info!("Failed to insert message {:?}", e));
        self.milestones.lock().await.insert(idx, milestone);
    }

    /// Remove a milestone from the tangle.
    pub async fn remove_milestone(&self, index: MilestoneIndex) {
        self.milestones.lock().await.remove(&index);
    }

    async fn pull_milestone(&self, idx: MilestoneIndex) -> Option<MessageId> {
        if let Some(milestone) = self.storage_get_milestone(&idx).unwrap_or_else(|e| {
            info!("Failed to insert message {:?}", e);
            None
        }) {
            let message_id = *self
                .milestones
                .lock()
                .await
                .entry(idx)
                .or_insert(milestone)
                .message_id();

            Some(message_id)
        } else {
            None
        }
    }

    /// Get the milestone from the tangle that corresponds to the given milestone index.
    pub async fn get_milestone(&self, index: MilestoneIndex) -> Option<Milestone> {
        self.milestones.lock().await.get(&index).cloned()
    }

    /// Get the message associated with the given milestone index from the tangle.
    pub async fn get_milestone_message(&self, index: MilestoneIndex) -> Option<MessageRef> {
        // TODO: use combinator instead of match
        match self.get_milestone_message_id(index).await {
            None => None,
            Some(ref hash) => self.get(hash).await,
        }
    }

    /// Get the message ID associated with the given milestone index from the tangle.
    pub async fn get_milestone_message_id(&self, index: MilestoneIndex) -> Option<MessageId> {
        let message_id = self.milestones.lock().await.get(&index).map(|m| *m.message_id());

        // TODO: use combinator instead of match
        match message_id {
            Some(m) => Some(m),
            None => Some(self.pull_milestone(index).await?),
        }
    }

    /// Return whether the tangle contains the given milestone index.
    pub async fn contains_milestone(&self, idx: MilestoneIndex) -> bool {
        // Not using `||` as its first operand would keep the lock alive causing a deadlock with its second operand.
        if self.milestones.lock().await.contains_key(&idx) {
            return true;
        }
        self.pull_milestone(idx).await.is_some()
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
    pub async fn get_solid_entry_point_index(&self, sep: &SolidEntryPoint) -> Option<MilestoneIndex> {
        self.solid_entry_points.lock().await.get(sep).copied()
    }

    /// Add the given solid entry point to the given milestone index.
    pub async fn add_solid_entry_point(&self, sep: SolidEntryPoint, index: MilestoneIndex) {
        self.solid_entry_points.lock().await.insert(sep, index);
    }

    /// Returns a copy of all solid entry points.
    pub async fn get_solid_entry_points(&self) -> HashMap<SolidEntryPoint, MilestoneIndex> {
        self.solid_entry_points.lock().await.clone()
    }

    /// Removes the given solid entry point from the set of solid entry points.
    pub async fn remove_solid_entry_point(&self, sep: &SolidEntryPoint) {
        self.solid_entry_points.lock().await.remove(sep);
    }

    /// Clear all solid entry points.
    pub async fn clear_solid_entry_points(&self) {
        self.solid_entry_points.lock().await.clear();
    }

    /// Replaces all solid entry points.
    pub async fn replace_solid_entry_points(
        &self,
        new_seps: impl IntoIterator<Item = (SolidEntryPoint, MilestoneIndex)>,
    ) {
        let mut seps = self.solid_entry_points.lock().await;
        seps.clear();
        seps.extend(new_seps);
    }

    /// Returns whether the message associated with given solid entry point is a solid entry point.
    pub async fn is_solid_entry_point(&self, id: &MessageId) -> bool {
        self.solid_entry_points
            .lock()
            .await
            .contains_key(SolidEntryPoint::ref_cast(id))
    }

    /// Returns whether the message associated with the given message ID is solid.
    pub async fn is_solid_message(&self, id: &MessageId) -> bool {
        if self.is_solid_entry_point(id).await {
            true
        } else {
            self.get_metadata(id)
                .await
                .map(|metadata| metadata.flags().is_solid())
                .unwrap_or(false)
        }
    }

    /// Get the oldest milestone root snapshot index.
    pub async fn omrsi(&self, id: &MessageId) -> Option<IndexId> {
        match self.solid_entry_points.lock().await.get(SolidEntryPoint::ref_cast(id)) {
            Some(sep) => Some(IndexId::new(*sep, *id)),
            None => match self.get_metadata(id).await {
                Some(metadata) => metadata.omrsi(),
                None => None,
            },
        }
    }

    /// Get the youngest milestone root snapshot index.
    pub async fn ymrsi(&self, id: &MessageId) -> Option<IndexId> {
        match self.solid_entry_points.lock().await.get(SolidEntryPoint::ref_cast(id)) {
            Some(sep) => Some(IndexId::new(*sep, *id)),
            None => match self.get_metadata(id).await {
                Some(metadata) => metadata.ymrsi(),
                None => None,
            },
        }
    }

    /// Insert the given message ID and parents as a tip.
    pub async fn insert_tip(&self, message_id: MessageId, parents: Vec<MessageId>) {
        self.tip_pool.lock().await.insert(self, message_id, parents).await;
    }

    /// Update tip scores.
    pub async fn update_tip_scores(&self) {
        self.tip_pool.lock().await.update_scores(self).await;
    }

    /// Return messages that require approving.
    pub async fn get_messages_to_approve(&self) -> Option<Vec<MessageId>> {
        self.tip_pool.lock().await.choose_non_lazy_tips()
    }

    /// Reduce tips.
    pub async fn reduce_tips(&self) {
        self.tip_pool.lock().await.reduce_tips();
    }

    /// Return the number of non-lazy tips.
    pub async fn non_lazy_tips_num(&self) -> usize {
        self.tip_pool.lock().await.non_lazy_tips().len()
    }

    /// Change the maximum number of entries to store in the cache.
    fn resize(&self, len: usize) {
        self.max_len.store(len, Ordering::Relaxed);
    }

    /// Return a reference to the storage  used by this tangle.
    fn storage(&self) -> &B {
        &self.storage
    }

    async fn insert_inner(
        &self,
        message_id: MessageId,
        message: Message,
        metadata: MessageMetadata,
    ) -> Option<MessageRef> {
        let mut vertices = self.get_interned_vertices(&message_id).write().await;
        let vertex = vertices.entry(message_id).or_insert_with(Vertex::empty);

        let msg = if vertex.message().is_some() {
            None
        } else {
            let parents = message.parents().clone();

            vertex.insert_message_and_metadata(message, metadata);
            let msg = vertex.message().cloned();

            // Insert children for parents
            for &parent in parents.iter() {
                let children = vertices.entry(parent).or_insert_with(Vertex::empty);
                children.add_child(message_id);
            }

            msg
        };

        drop(vertices);

        self.len.fetch_add(1, Ordering::SeqCst);

        self.perform_eviction().await;

        msg
    }

    async fn get_inner(&self, message_id: &MessageId) -> Option<impl DerefMut<Target = Vertex> + '_> {
        self.get_interned_mut(message_id).await
    }

    /// Get the data of a vertex associated with the given `message_id`.
    async fn get_with<R>(&self, message_id: &MessageId, f: impl FnOnce(&mut Vertex) -> R) -> Option<R> {
        self.pull_message(message_id).await;

        self.get_inner(message_id).await.map(|mut v| f(&mut v))
    }

    /// Get the data of a vertex associated with the given `message_id`.
    pub async fn get(&self, message_id: &MessageId) -> Option<MessageRef> {
        self.get_with(message_id, |v| v.message().cloned()).await.flatten()
    }

    async fn contains_inner(&self, message_id: &MessageId) -> bool {
        self.get_interned(message_id)
            .await
            .map_or(false, |v| v.message().is_some())
    }

    /// Returns whether the message is stored in the Tangle.
    pub async fn contains(&self, message_id: &MessageId) -> bool {
        self.contains_inner(message_id).await || self.pull_message(message_id).await
    }

    /// Get the metadata of a vertex associated with the given `message_id`.
    pub async fn get_metadata(&self, message_id: &MessageId) -> Option<MessageMetadata> {
        self.get_with(message_id, |v| v.metadata().cloned()).await.flatten()
    }

    /// Get the metadata of a vertex associated with the given `message_id`.
    pub async fn get_vertex(&self, message_id: &MessageId) -> Option<impl Deref<Target = Vertex> + '_> {
        self.pull_message(message_id).await;

        self.get_inner(message_id).await
    }

    /// Updates the metadata of a vertex.
    pub async fn update_metadata<R, Update>(&self, message_id: &MessageId, update: Update) -> Option<R>
    where
        Update: FnOnce(&mut MessageMetadata) -> R,
    {
        self.pull_message(message_id).await;
        let mut vertices = self.get_interned_vertices(message_id).write().await;

        if let Some(vertex) = vertices.get_mut(message_id) {
            let r = vertex.metadata_mut().map(|m| update(m));

            if let Some((msg, meta)) = vertex.message_and_metadata() {
                let (msg, meta) = ((&**msg).clone(), *meta);

                drop(vertices);

                self.storage_insert(*message_id, msg, meta)
                    .unwrap_or_else(|e| info!("Failed to update metadata for message {:?}", e));
            }

            r
        } else {
            None
        }
    }

    async fn children_inner(&self, message_id: &MessageId) -> Option<impl Deref<Target = Vec<MessageId>> + '_> {
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

        let vertices = self.get_interned_vertices(message_id).read().await;
        let v = vertices
            .get(message_id)
            // Skip approver lists that are not exhaustive
            .filter(|v| v.children_exhaustive());

        let children = match v {
            Some(v) => {
                let children = v.children().to_vec();
                drop(vertices);
                children
            }
            None => {
                drop(vertices);
                let to_insert = match self.storage_fetch_approvers(message_id) {
                    Err(e) => {
                        info!("Failed to update approvers for message message {:?}", e);
                        Vec::new()
                    }
                    Ok(None) => Vec::new(),
                    Ok(Some(approvers)) => approvers,
                };

                let mut vertices = self.get_interned_vertices(message_id).write().await;
                let v = vertices.entry(*message_id).or_insert_with(Vertex::empty);

                self.len.fetch_add(1, Ordering::SeqCst);

                // We've just fetched approvers from the database, so we have all the information available to us now.
                // Therefore, the approvers list is exhaustive (i.e: it contains all knowledge we have).
                v.set_exhaustive();

                for child in to_insert {
                    v.add_child(child);
                }

                v.children().to_vec()
            }
        };

        Some(Wrapper {
            children,
            phantom: PhantomData,
        })
    }

    /// Returns the children of a vertex, if we know about them.
    pub async fn get_children(&self, message_id: &MessageId) -> Option<Vec<MessageId>> {
        // Effectively atomic
        self.children_inner(message_id).await.map(|approvers| approvers.clone())
    }

    #[cfg(test)]
    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    // Attempts to pull the message from the storage, returns true if successful.
    async fn pull_message(&self, message_id: &MessageId) -> bool {
        // If the tangle already contains the message, do no more work
        if self.contains_inner(message_id).await {
            true
        } else if let Ok(Some((msg, metadata))) = self.storage_get(message_id) {
            self.insert_inner(*message_id, msg, metadata).await;

            true
        } else {
            false
        }
    }

    async fn perform_eviction(&self) {
        let max_len = self.max_len.load(Ordering::Relaxed);
        let mut len = self.len.load(Ordering::Relaxed);

        if len > max_len {
            while len > ((1.0 - CACHE_THRESHOLD_FACTOR) * max_len as f64) as usize {
                let idx = rand::thread_rng().gen_range(0..PARTITION_LENGTH) as usize;
                let mut vertices = self.vertices[idx].write().await;

                if let Some(key) = vertices.iter().next().map(|(k, _)| k).copied() {
                    if vertices.remove(&key).is_some() {
                        len = self.len.fetch_sub(1, Ordering::SeqCst) - 1;
                    }
                }
            }
        }
    }
}

impl<B: StorageBackend> Tangle<B> {
    fn storage_get_milestone(&self, idx: &MilestoneIndex) -> Result<Option<Milestone>, B::Error> {
        trace!("Attempted to fetch milestone {:?}", idx);
        self.storage().fetch(idx)
    }

    fn storage_insert_milestone(&self, idx: MilestoneIndex, milestone: &Milestone) -> Result<(), B::Error> {
        trace!("Attempted to insert milestone {:?}", idx);
        self.storage().insert(&idx, milestone)?;
        Ok(())
    }

    fn storage_get(&self, id: &MessageId) -> Result<Option<(Message, MessageMetadata)>, B::Error> {
        trace!("Attempted to fetch message {:?}", id);
        let msg = self.storage.fetch(id)?;
        let meta = self.storage.fetch(id)?;
        Ok(msg.zip(meta))
    }

    fn storage_insert(&self, id: MessageId, tx: Message, metadata: MessageMetadata) -> Result<(), B::Error> {
        trace!("Attempted to insert message {:?}", id);
        self.storage.insert(&id, &tx)?;
        self.storage.insert(&id, &metadata)?;
        Ok(())
    }

    fn storage_fetch_approvers(&self, id: &MessageId) -> Result<Option<Vec<MessageId>>, B::Error> {
        trace!("Attempted to fetch approvers for message {:?}", id);
        self.storage.fetch(id)
    }

    fn storage_insert_approver(&self, id: MessageId, approver: MessageId) -> Result<(), B::Error> {
        trace!("Attempted to insert approver for message {:?}", id);
        self.storage.insert(&(id, approver), &())
    }
}
