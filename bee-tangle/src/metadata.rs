// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::flags::Flags;

use bee_common::packable::{OptionError, Packable, Read, Write};
use bee_ledger::types::ConflictReason;
use bee_message::{milestone::MilestoneIndex, MessageId};

use serde::Serialize;

use std::{
    cmp::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

/// Metadata associated with a tangle message.
#[derive(Copy, Clone, Default, Debug, Serialize)]
pub struct MessageMetadata {
    flags: Flags,
    milestone_index: Option<MilestoneIndex>,
    arrival_timestamp: u64,
    solidification_timestamp: u64,
    reference_timestamp: u64,
    omrsi: Option<IndexId>,
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
    /// An IO error occurred.
    Io(std::io::Error),
    /// A packing error occurred.
    OptionIndex(<Option<MilestoneIndex> as Packable>::Error),
    /// A packing error occurred.
    OptionIndexId(<Option<IndexId> as Packable>::Error),
}

impl From<std::io::Error> for MessageMetadataError {
    fn from(error: std::io::Error) -> Self {
        MessageMetadataError::Io(error)
    }
}

impl From<OptionError<std::io::Error>> for MessageMetadataError {
    fn from(error: OptionError<std::io::Error>) -> Self {
        MessageMetadataError::OptionIndex(error)
    }
}

impl From<OptionError<IndexIdError>> for MessageMetadataError {
    fn from(error: OptionError<IndexIdError>) -> Self {
        MessageMetadataError::OptionIndexId(error)
    }
}

impl Packable for MessageMetadata {
    type Error = MessageMetadataError;

    fn packed_len(&self) -> usize {
        self.flags.packed_len()
            + self.milestone_index.packed_len()
            + self.arrival_timestamp.packed_len()
            + self.solidification_timestamp.packed_len()
            + self.reference_timestamp.packed_len()
            + self.omrsi.packed_len()
            + self.ymrsi.packed_len()
            + 0u8.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.flags.pack(writer)?;
        self.milestone_index.pack(writer)?;
        self.arrival_timestamp.pack(writer)?;
        self.solidification_timestamp.pack(writer)?;
        self.reference_timestamp.pack(writer)?;
        self.omrsi.pack(writer)?;
        self.ymrsi.pack(writer)?;
        (self.conflict as u8).pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self {
            flags: Flags::unpack_inner::<R, CHECK>(reader)?,
            milestone_index: Option::<MilestoneIndex>::unpack_inner::<R, CHECK>(reader)?,
            arrival_timestamp: u64::unpack_inner::<R, CHECK>(reader)?,
            solidification_timestamp: u64::unpack_inner::<R, CHECK>(reader)?,
            reference_timestamp: u64::unpack_inner::<R, CHECK>(reader)?,
            omrsi: Option::<IndexId>::unpack_inner::<R, CHECK>(reader)?,
            ymrsi: Option::<IndexId>::unpack_inner::<R, CHECK>(reader)?,
            conflict: ConflictReason::unpack_inner::<R, CHECK>(reader)?,
        })
    }
}

/// A type used to associate two particular interesting Cone Root Indexes with a message in the Tangle, i.e. the Oldest
/// Cone Root Index (OCRI), and the Youngest Cone Root Index (YCRI)
#[derive(Clone, Copy, Debug, Serialize)]
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

impl Packable for IndexId {
    type Error = IndexIdError;

    fn packed_len(&self) -> usize {
        self.0.packed_len() + self.1.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;
        self.1.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let index = MilestoneIndex::unpack_inner::<R, CHECK>(reader)?;
        let id = MessageId::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self(index, id))
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
