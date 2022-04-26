// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    cmp::Ordering,
    convert::Infallible,
    time::{SystemTime, UNIX_EPOCH},
};

use bee_message::{
    milestone::MilestoneIndex,
    semantic::{ConflictError, ConflictReason},
    MessageId,
};
use packable::Packable;
use serde::Serialize;

use crate::flags::Flags;

/// Metadata associated with a tangle message.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Serialize, Packable)]
#[packable(unpack_error = MessageMetadataError)]
pub struct MessageMetadata {
    flags: Flags,
    #[packable(unpack_error_with = MessageMetadataError::OptionIndex)]
    milestone_index: Option<MilestoneIndex>,
    arrival_timestamp: u64,
    solidification_timestamp: u64,
    reference_timestamp: u32,
    #[packable(unpack_error_with = MessageMetadataError::OptionIndexId)]
    omrsi_and_ymrsi: Option<(IndexId, IndexId)>,
    #[packable(unpack_error_with = MessageMetadataError::Conflict)]
    conflict: ConflictReason,
}

impl MessageMetadata {
    /// Create a new instance of a message's metadata.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        flags: Flags,
        milestone_index: Option<MilestoneIndex>,
        arrival_timestamp: u64,
        solidification_timestamp: u64,
        reference_timestamp: u32,
        omrsi_and_ymrsi: Option<(IndexId, IndexId)>,
        conflict: ConflictReason,
    ) -> Self {
        Self {
            flags,
            milestone_index,
            arrival_timestamp,
            solidification_timestamp,
            reference_timestamp,
            omrsi_and_ymrsi,
            conflict,
        }
    }

    /// Create metadata that corresponds to a just-arrived message using the current system time.
    pub fn arrived() -> Self {
        Self {
            arrival_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock may have gone backwards")
                .as_millis() as u64,
            ..Self::default()
        }
    }

    /// Get the flags associated with this metadata.
    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    /// Get a mutable reference to the flags associated with this metadata.
    pub fn flags_mut(&mut self) -> &mut Flags {
        &mut self.flags
    }

    /// Get the milestone index of this message.
    pub fn milestone_index(&self) -> Option<MilestoneIndex> {
        self.milestone_index
    }

    /// Set the milestone index of this message.
    pub fn set_milestone_index(&mut self, index: MilestoneIndex) {
        self.milestone_index = Some(index);
    }

    /// Get the arrival timestamp (seconds from the unix epoch) of this message.
    pub fn arrival_timestamp(&self) -> u64 {
        self.arrival_timestamp
    }

    /// Get the solidification timestamp (seconds from the unix epoch) of this message.
    pub fn solidification_timestamp(&self) -> u64 {
        self.solidification_timestamp
    }

    /// Get the oldest and youngest message root snapshot index of this message.
    pub fn omrsi_and_ymrsi(&self) -> Option<(IndexId, IndexId)> {
        self.omrsi_and_ymrsi
    }

    /// Set the oldest and youngest message root snapshot index of this message.
    pub fn set_omrsi_and_ymrsi(&mut self, omrsi: IndexId, ymrsi: IndexId) {
        self.omrsi_and_ymrsi = Some((omrsi, ymrsi));
    }

    /// Update the oldest and youngest message root snapshot index of this message if they have
    /// been set already.
    pub fn update_omrsi_and_ymrsi(&mut self, f: impl FnOnce(&mut IndexId, &mut IndexId)) {
        if let Some((omrsi, ymrsi)) = self.omrsi_and_ymrsi.as_mut() {
            f(omrsi, ymrsi);
        }
    }

    /// Get the reference timestamp (seconds from the unix epoch) of this message.
    pub fn reference_timestamp(&self) -> u32 {
        self.reference_timestamp
    }

    /// Mark this message as solid at the current system time.
    pub fn mark_solid(&mut self) {
        self.flags.set_solid(true);
        self.solidification_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis() as u64;
    }

    /// Reference this message with the given timestamp.
    pub fn reference(&mut self, timestamp: u32) {
        self.flags.set_referenced(true);
        self.reference_timestamp = timestamp;
    }

    /// Get the conflict state of this message.
    pub fn conflict(&self) -> ConflictReason {
        self.conflict
    }

    /// Set the conflict state of this message.
    pub fn set_conflict(&mut self, conflict: ConflictReason) {
        self.conflict = conflict;
    }
}

/// An error that may occur when manipulating message metadata.
#[derive(Debug)]
pub enum MessageMetadataError {
    /// A packing error occurred.
    OptionIndex(<Option<MilestoneIndex> as Packable>::UnpackError),
    /// A packing error occurred.
    OptionIndexId(<Option<IndexId> as Packable>::UnpackError),
    /// An error relating to a conflict reason occurred.
    Conflict(ConflictError),
}

impl From<Infallible> for MessageMetadataError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

/// A type used to associate two particular interesting Cone Root Indexes with a message in the Tangle, i.e. the Oldest
/// Cone Root Index (OCRI), and the Youngest Cone Root Index (YCRI)
#[derive(Clone, Copy, Debug, Serialize, packable::Packable)]
pub struct IndexId(MilestoneIndex, MessageId);

impl IndexId {
    /// Create a new `IndexId`.
    pub fn new(index: MilestoneIndex, id: MessageId) -> Self {
        Self(index, id)
    }

    /// Get the milestone index of this `IndexId`.
    pub fn index(&self) -> MilestoneIndex {
        self.0
    }

    /// Get the message ID of this `IndexId`.
    pub fn id(&self) -> MessageId {
        self.1
    }

    /// Update this `IndexId` with a new milestone index.
    pub fn set_index(&mut self, index: MilestoneIndex) {
        self.0 = index;
    }
}

impl Ord for IndexId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for IndexId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Eq for IndexId {}

impl PartialEq for IndexId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// An error that may occur when manipulating message indices.
#[derive(Debug)]
pub enum IndexIdError {
    /// An IO error occurred.
    Io(std::io::Error),
    /// An message-related error occurred.
    MessageId(bee_message::Error),
}

impl From<std::io::Error> for IndexIdError {
    fn from(error: std::io::Error) -> Self {
        IndexIdError::Io(error)
    }
}

impl From<bee_message::Error> for IndexIdError {
    fn from(error: bee_message::Error) -> Self {
        IndexIdError::MessageId(error)
    }
}
