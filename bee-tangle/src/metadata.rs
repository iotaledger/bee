// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    conflict::{ConflictError, ConflictReason},
    flags::Flags,
};

use bee_message::{milestone::MilestoneIndex, MessageId};
use bee_packable::Packable;

use serde::Serialize;

use std::{
    cmp::Ordering,
    convert::Infallible,
    time::{SystemTime, UNIX_EPOCH},
};

/// Metadata associated with a tangle message.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Serialize, Packable)]
#[packable(unpack_error = MessageMetadataError)]
pub struct MessageMetadata {
    flags: Flags,
    #[packable(unpack_error_with = MessageMetadataError::OptionIndex)]
    milestone_index: Option<MilestoneIndex>,
    arrival_timestamp: u64,
    solidification_timestamp: u64,
    reference_timestamp: u64,
    #[packable(unpack_error_with = MessageMetadataError::OptionIndexId)]
    omrsi: Option<IndexId>,
    #[packable(unpack_error_with = MessageMetadataError::OptionIndexId)]
    ymrsi: Option<IndexId>,
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
        reference_timestamp: u64,
        omrsi: Option<IndexId>,
        ymrsi: Option<IndexId>,
        conflict: ConflictReason,
    ) -> Self {
        Self {
            flags,
            milestone_index,
            arrival_timestamp,
            solidification_timestamp,
            reference_timestamp,
            omrsi,
            ymrsi,
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

    /// Get the oldest message root snapshot index of this message.
    pub fn omrsi(&self) -> Option<IndexId> {
        self.omrsi
    }

    /// Set the oldest message root snapshot index of this message.
    pub fn set_omrsi(&mut self, omrsi: IndexId) {
        self.omrsi = Some(omrsi);
    }

    /// Get the youngest message root snapshot index of this message.
    pub fn ymrsi(&self) -> Option<IndexId> {
        self.ymrsi
    }

    /// Set the youngest message root snapshot index of this message.
    pub fn set_ymrsi(&mut self, ymrsi: IndexId) {
        self.ymrsi = Some(ymrsi);
    }

    /// Get the reference timestamp (seconds from the unix epoch) of this message.
    pub fn reference_timestamp(&self) -> u64 {
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
    pub fn reference(&mut self, timestamp: u64) {
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

impl From<ConflictError> for MessageMetadataError {
    fn from(err: ConflictError) -> Self {
        Self::Conflict(err)
    }
}

impl From<Infallible> for MessageMetadataError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

/// A type used to associate two particular interesting Cone Root Indexes with a message in the Tangle, i.e. the Oldest
/// Cone Root Index (OCRI), and the Youngest Cone Root Index (YCRI)
#[derive(Clone, Copy, Debug, Serialize, Packable)]
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
