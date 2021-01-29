// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::flags::Flags;

use bee_common::packable::{OptionError, Packable, Read, Write};
use bee_message::{milestone::MilestoneIndex, MessageId};

use std::{
    cmp::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

// TODO Should it really be copy ?
#[derive(Copy, Clone, Default, Debug)]
pub struct MessageMetadata {
    flags: Flags,
    milestone_index: Option<MilestoneIndex>,
    arrival_timestamp: u64,
    solidification_timestamp: u64,
    confirmation_timestamp: u64,
    otrsi: Option<IndexId>,
    ytrsi: Option<IndexId>,
    conflict: u8,
}

impl MessageMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        flags: Flags,
        milestone_index: Option<MilestoneIndex>,
        arrival_timestamp: u64,
        solidification_timestamp: u64,
        confirmation_timestamp: u64,
        otrsi: Option<IndexId>,
        ytrsi: Option<IndexId>,
        conflict: u8,
    ) -> Self {
        Self {
            flags,
            milestone_index,
            arrival_timestamp,
            solidification_timestamp,
            confirmation_timestamp,
            otrsi,
            ytrsi,
            conflict,
        }
    }

    pub fn arrived() -> Self {
        Self {
            arrival_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock may have gone backwards")
                .as_millis() as u64,
            ..Self::default()
        }
    }

    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    pub fn flags_mut(&mut self) -> &mut Flags {
        &mut self.flags
    }

    pub fn milestone_index(&self) -> Option<MilestoneIndex> {
        self.milestone_index
    }

    pub fn set_milestone_index(&mut self, index: MilestoneIndex) {
        self.milestone_index = Some(index);
    }

    pub fn arrival_timestamp(&self) -> u64 {
        self.arrival_timestamp
    }

    pub fn solidification_timestamp(&self) -> u64 {
        self.solidification_timestamp
    }

    pub fn otrsi(&self) -> Option<IndexId> {
        self.otrsi
    }

    pub fn set_otrsi(&mut self, otrsi: IndexId) {
        self.otrsi = Some(otrsi);
    }

    pub fn ytrsi(&self) -> Option<IndexId> {
        self.ytrsi
    }

    pub fn set_ytrsi(&mut self, ytrsi: IndexId) {
        self.ytrsi = Some(ytrsi);
    }

    pub fn confirmation_timestamp(&self) -> u64 {
        self.confirmation_timestamp
    }

    pub fn solidify(&mut self) {
        self.flags.set_solid(true);
        self.solidification_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis() as u64;
    }

    pub fn confirm(&mut self, timestamp: u64) {
        self.flags.set_confirmed(true);
        self.confirmation_timestamp = timestamp;
    }

    pub fn conflict(&self) -> u8 {
        self.conflict
    }

    pub fn set_conflict(&mut self, conflict: u8) {
        self.conflict = conflict;
    }
}

#[derive(Debug)]
pub enum MessageMetadataError {
    Io(std::io::Error),
    OptionIndex(<Option<MilestoneIndex> as Packable>::Error),
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
            + self.confirmation_timestamp.packed_len()
            + self.otrsi.packed_len()
            + self.ytrsi.packed_len()
            + self.conflict.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.flags.pack(writer)?;
        self.milestone_index.pack(writer)?;
        self.arrival_timestamp.pack(writer)?;
        self.solidification_timestamp.pack(writer)?;
        self.confirmation_timestamp.pack(writer)?;
        self.otrsi.pack(writer)?;
        self.ytrsi.pack(writer)?;
        self.conflict.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self {
            flags: Flags::unpack(reader)?,
            milestone_index: Option::<MilestoneIndex>::unpack(reader)?,
            arrival_timestamp: u64::unpack(reader)?,
            solidification_timestamp: u64::unpack(reader)?,
            confirmation_timestamp: u64::unpack(reader)?,
            otrsi: Option::<IndexId>::unpack(reader)?,
            ytrsi: Option::<IndexId>::unpack(reader)?,
            conflict: u8::unpack(reader)?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct IndexId(pub MilestoneIndex, pub MessageId);

impl IndexId {
    pub fn update(&mut self, index: MilestoneIndex) {
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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(MilestoneIndex::unpack(reader)?, MessageId::unpack(reader)?))
    }
}

#[derive(Debug)]
pub enum IndexIdError {
    Io(std::io::Error),
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
