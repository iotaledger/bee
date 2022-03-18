// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::TangleConfig,
    metadata::{IndexId, MessageMetadata},
    solid_entry_point::SolidEntryPoint,
    storage::StorageBackend,
    urts::UrtsTipPool,
};

use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    Message, MessageId,
};
use bee_runtime::resource::ResourceHandle;

use hashbrown::HashMap;
use log::warn;
use ref_cast::RefCast;
use tokio::sync::Mutex;

use std::sync::atomic::{AtomicU32, Ordering};

const SYNCED_THRESHOLD: u32 = 2;
const CONFIRMED_THRESHOLD: u32 = 2;

/// A Tangle wrapper designed to encapsulate milestone state.
pub struct Tangle<B> {
    config: TangleConfig,
    storage: ResourceHandle<B>,
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
            storage,
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

    /// Insert a message into the tangle without overwriting its metadata if it already exists.
    pub fn insert(&self, message: &Message, message_id: &MessageId, metadata: &MessageMetadata) {
        self.storage
            .insert(message_id, message)
            .ok()
            .and_then(|()| {
                self.storage.insert_strict(message_id, metadata).ok()?;

                let message_id = *message_id;
                for &parent in message.parents().iter() {
                    self.storage
                        .insert(&(parent, message_id), &())
                        .unwrap_or_else(|e| warn!("Failed to update approvers for message {:?}", e));
                }
                Some(())
            })
            .unwrap_or_default()
    }

    /// Add a milestone to the tangle.
    pub fn add_milestone(&self, idx: MilestoneIndex, milestone: Milestone) {
        let index = IndexId::new(idx, *milestone.message_id());

        self.update_metadata(milestone.message_id(), |metadata| {
            metadata.flags_mut().set_milestone(true);
            metadata.set_milestone_index(idx);
            metadata.set_omrsi_and_ymrsi(index, index);
        });
        self.storage
            .insert(&idx, &milestone)
            .unwrap_or_else(|e| warn!("Failed to insert milestone message {:?}", e));
    }

    /// Get the milestone from the tangle that corresponds to the given milestone index.
    pub fn get_milestone(&self, index: MilestoneIndex) -> Option<Milestone> {
        self.storage.fetch(&index).unwrap_or_else(|e| {
            warn!("Failed to fetch milestone {:?}", e);
            None
        })
    }

    /// Get the message associated with the given milestone index from the tangle.
    pub fn get_milestone_message(&self, index: MilestoneIndex) -> Option<Message> {
        self.get_milestone_message_id(index).and_then(|hash| self.get(&hash))
    }

    /// Get the message ID associated with the given milestone index from the tangle.
    pub fn get_milestone_message_id(&self, index: MilestoneIndex) -> Option<MessageId> {
        self.get_milestone(index).map(|m| *m.message_id())
    }

    /// Return whether the tangle contains the given milestone index.
    pub fn contains_milestone(&self, index: MilestoneIndex) -> bool {
        self.storage.exist(&index).unwrap_or_default()
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
                .map(|metadata| metadata.flags().is_solid())
                .unwrap_or(false)
        }
    }

    /// Get the oldest and youngest milestone root snapshot index.
    pub async fn omrsi_and_ymrsi(&self, id: &MessageId) -> Option<(IndexId, IndexId)> {
        match self.solid_entry_points.lock().await.get(SolidEntryPoint::ref_cast(id)) {
            Some(sep) => {
                let index = IndexId::new(*sep, *id);
                Some((index, index))
            }
            None => self.get_metadata(id).and_then(|metadata| metadata.omrsi_and_ymrsi()),
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

    /// Get the data of a vertex associated with the given `message_id`.
    pub fn get(&self, message_id: &MessageId) -> Option<Message> {
        self.storage.fetch(message_id).unwrap_or_default()
    }

    /// Get the data and metadata of a vertex associated with the given `message_id`.
    pub fn get_message_and_metadata(&self, message_id: &MessageId) -> Option<(Message, MessageMetadata)> {
        let msg = self.storage.fetch(message_id).unwrap_or_default()?;
        let meta = self.storage.fetch(message_id).unwrap_or_default()?;

        Some((msg, meta))
    }

    /// Returns whether the message is stored in the Tangle.
    pub fn contains(&self, message_id: &MessageId) -> bool {
        self.storage.exist(message_id).unwrap_or_default()
    }

    /// Get the metadata of a vertex associated with the given `message_id`.
    pub fn get_metadata(&self, message_id: &MessageId) -> Option<MessageMetadata> {
        self.storage.fetch(message_id).unwrap_or_default()
    }

    /// Updates the metadata of a vertex.
    pub fn update_metadata<R>(
        &self,
        message_id: &MessageId,
        update: impl FnOnce(&mut MessageMetadata) -> R + Copy,
    ) -> Option<R> {
        let mut output = None;

        self.storage
            .update(message_id, |metadata| output = Some(update(metadata)))
            .unwrap_or_default();

        output
    }

    /// Returns the children of a vertex, if we know about them.
    pub fn get_children(&self, message_id: &MessageId) -> Option<Vec<MessageId>> {
        self.storage.fetch(message_id).unwrap_or_default()
    }
}
