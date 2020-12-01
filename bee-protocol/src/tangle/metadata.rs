// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{milestone::MilestoneIndex, tangle::Flags};

use bee_common::packable::{OptionError, Packable, Read, Write};

use std::time::{SystemTime, UNIX_EPOCH};

// TODO Should it really be copy ?
#[derive(Copy, Clone, Default)]
pub struct MessageMetadata {
    flags: Flags,
    milestone_index: MilestoneIndex,
    arrival_timestamp: u64,
    solidification_timestamp: u64,
    confirmation_timestamp: u64,
    cone_index: Option<MilestoneIndex>, /* maybe merge milestone_index and cone_index; keep it like that for now to
                                         * avoid conflicts; */
    otrsi: Option<MilestoneIndex>,
    ytrsi: Option<MilestoneIndex>,
}

impl MessageMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        flags: Flags,
        milestone_index: MilestoneIndex,
        arrival_timestamp: u64,
        solidification_timestamp: u64,
        confirmation_timestamp: u64,
        cone_index: Option<MilestoneIndex>,
        otrsi: Option<MilestoneIndex>,
        ytrsi: Option<MilestoneIndex>,
    ) -> Self {
        Self {
            flags,
            milestone_index,
            arrival_timestamp,
            solidification_timestamp,
            confirmation_timestamp,
            cone_index,
            otrsi,
            ytrsi,
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

    pub fn milestone_index(&self) -> MilestoneIndex {
        self.milestone_index
    }

    pub fn set_milestone_index(&mut self, index: MilestoneIndex) {
        self.milestone_index = index;
    }

    pub fn arrival_timestamp(&self) -> u64 {
        self.arrival_timestamp
    }

    pub fn solidification_timestamp(&self) -> u64 {
        self.solidification_timestamp
    }

    pub fn set_solidification_timestamp(&mut self, timestamp: u64) {
        self.solidification_timestamp = timestamp;
    }

    pub fn cone_index(&self) -> Option<MilestoneIndex> {
        self.cone_index
    }

    pub fn set_cone_index(&mut self, cone_index: MilestoneIndex) {
        self.cone_index = Some(cone_index);
    }

    pub fn otrsi(&self) -> Option<MilestoneIndex> {
        self.otrsi
    }

    pub fn set_otrsi(&mut self, otrsi: MilestoneIndex) {
        self.otrsi = Some(otrsi);
    }

    pub fn ytrsi(&self) -> Option<MilestoneIndex> {
        self.ytrsi
    }

    pub fn set_ytrsi(&mut self, ytrsi: MilestoneIndex) {
        self.ytrsi = Some(ytrsi);
    }

    pub fn confirmation_timestamp(&self) -> u64 {
        self.confirmation_timestamp
    }

    pub fn set_confirmation_timestamp(&mut self, timestamp: u64) {
        self.confirmation_timestamp = timestamp;
    }

    pub fn solidify(&mut self) {
        self.flags.set_solid(true);
        self.solidification_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis() as u64;
    }

    pub fn confirm(&mut self) {
        self.flags.set_confirmed(true);
        self.confirmation_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis() as u64;
    }
}

#[derive(Debug)]
pub enum MessageMetadataError {
    Io(std::io::Error),
    OptionIndex(<Option<MilestoneIndex> as Packable>::Error),
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

impl Packable for MessageMetadata {
    type Error = MessageMetadataError;

    fn packed_len(&self) -> usize {
        self.flags.packed_len()
            + self.milestone_index.packed_len()
            + self.arrival_timestamp.packed_len()
            + self.solidification_timestamp.packed_len()
            + self.confirmation_timestamp.packed_len()
            + self.cone_index.packed_len()
            + self.otrsi.packed_len()
            + self.ytrsi.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.flags.pack(writer)?;
        self.milestone_index.pack(writer)?;
        self.arrival_timestamp.pack(writer)?;
        self.solidification_timestamp.pack(writer)?;
        self.confirmation_timestamp.pack(writer)?;
        self.cone_index.pack(writer)?;
        self.otrsi.pack(writer)?;
        self.ytrsi.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let flags = Flags::unpack(reader)?;
        let milestone_index = MilestoneIndex::unpack(reader)?;
        let arrival_timestamp = u64::unpack(reader)?;
        let solidification_timestamp = u64::unpack(reader)?;
        let confirmation_timestamp = u64::unpack(reader)?;
        let cone_index = Option::<MilestoneIndex>::unpack(reader)?;
        let otrsi = Option::<MilestoneIndex>::unpack(reader)?;
        let ytrsi = Option::<MilestoneIndex>::unpack(reader)?;

        Ok(Self {
            flags,
            milestone_index,
            arrival_timestamp,
            solidification_timestamp,
            confirmation_timestamp,
            cone_index,
            otrsi,
            ytrsi,
        })
    }
}
