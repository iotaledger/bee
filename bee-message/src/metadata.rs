// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::fpc::Opinion, MessageUnpackError};

use bee_packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use bitflags::bitflags;

use core::convert::Infallible;

bitflags! {
    /// Flags representing the state of a message.
    #[derive(Default)]
    #[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
    pub struct Flags: u8 {
        /// Denotes whether a message is solid, i.e. its past cone is known.
        const SOLID = 0b0000_0001;
        /// Denotes whether a message was scheduled by the scheduler.
        const SCHEDULED = 0b0000_0010;
        /// Denotes whether a message was booked and therefore is part of the local Tangle.
        const BOOKED = 0b0000_0100;
        /// Denotes whether a message is eligible, i.e. it's timestamp is good.
        const ELIGIBLE = 0b0000_1000;
        /// Denotes whether a message has been deemed invalid, i.e. it or its parents do not pass all checks from
        /// filters to message booker.
        const INVALID = 0b0001_0000;
    }
}

impl Flags {
    /// Returns the solid flag.
    pub fn is_solid(&self) -> bool {
        self.contains(Flags::SOLID)
    }

    /// Sets the solid flag.
    pub fn set_solid(&mut self, is_solid: bool) {
        self.set(Flags::SOLID, is_solid);
    }

    /// Returns the scheduled flag.
    pub fn is_scheduled(&self) -> bool {
        self.contains(Flags::SCHEDULED)
    }

    /// Sets the scheduled flag.
    pub fn set_scheduled(&mut self, is_scheduled: bool) {
        self.set(Flags::SCHEDULED, is_scheduled);
    }

    /// Returns the booked flag.
    pub fn is_booked(&self) -> bool {
        self.contains(Flags::BOOKED)
    }

    /// Sets the booked flag.
    pub fn set_booked(&mut self, is_booked: bool) {
        self.set(Flags::BOOKED, is_booked);
    }

    /// Returns the eligible flag.
    pub fn is_eligible(&self) -> bool {
        self.contains(Flags::ELIGIBLE)
    }

    /// Sets the eligible flag.
    pub fn set_eligible(&mut self, is_eligible: bool) {
        self.set(Flags::ELIGIBLE, is_eligible);
    }

    /// Returns the invalid flag.
    pub fn is_invalid(&self) -> bool {
        self.contains(Flags::INVALID)
    }

    /// Sets the invalid flag.
    pub fn set_invalid(&mut self, is_invalid: bool) {
        self.set(Flags::INVALID, is_invalid);
    }
}

impl Packable for Flags {
    type UnpackError = Infallible;

    fn packed_len(&self) -> usize {
        self.bits().packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.bits().pack(packer)
    }

    fn unpack<U: Unpacker, const CHECK: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        Ok(Self::from_bits_truncate(u8::unpack::<_, CHECK>(unpacker).infallible()?))
    }
}

/// Additional data that describes the local perception of a message which is not part of the Tangle.
#[derive(Clone, Default, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MessageMetadata {
    /// The flags describing the message state.
    flags: Flags,
    /// The local time the message was received by the node.
    received_timestamp: u64,
    /// The local time the message got solid.
    solidification_timestamp: u64,
    /// The branch ID of the message, i.e. the part of the Tangle where the message is located.
    branch_id: [u8; 32],
    /// Contains the node's opinion on the issuing timestamp of a message.
    opinion: Opinion,
}

impl MessageMetadata {
    /// Creates a new [`MessageMetadata`].
    pub fn new(received_timestamp: u64) -> Self {
        Self {
            received_timestamp,
            ..Self::default()
        }
    }

    /// Returns the flags of the [`MessageMetadata`].
    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    /// Returns mutable access to the flags of the [`MessageMetadata`].
    pub fn flags_mut(&mut self) -> &mut Flags {
        &mut self.flags
    }

    /// Returns the received timestamp of the [`MessageMetadata`].
    pub fn received_timestamp(&self) -> u64 {
        self.received_timestamp
    }

    /// Sets the received timestamp of the [`MessageMetadata`].
    pub fn set_received_timestamp(&mut self, received_timestamp: u64) {
        self.received_timestamp = received_timestamp;
    }

    /// Returns the solidification timestamp of the [`MessageMetadata`].
    pub fn solidification_timestamp(&self) -> u64 {
        self.solidification_timestamp
    }

    /// Sets the solidification timestamp of the [`MessageMetadata`].
    pub fn set_solidification_timestamp(&mut self, solidification_timestamp: u64) {
        self.solidification_timestamp = solidification_timestamp;
    }

    /// Returns the branch ID of the [`MessageMetadata`].
    pub fn branch_id(&self) -> &[u8; 32] {
        &self.branch_id
    }

    /// Sets the branch ID of the [`MessageMetadata`].
    pub fn set_branch_id(&mut self, branch_id: [u8; 32]) {
        self.branch_id = branch_id;
    }

    /// Returns the opinion of the [`MessageMetadata`].
    pub fn opinion(&self) -> &Opinion {
        &self.opinion
    }

    /// Sets the opinion of the [`MessageMetadata`].
    pub fn set_opinion(&mut self, opinion: Opinion) {
        self.opinion = opinion;
    }
}

impl Packable for MessageMetadata {
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.flags.packed_len()
            + self.received_timestamp.packed_len()
            + self.solidification_timestamp.packed_len()
            + self.branch_id.packed_len()
            + self.opinion.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.flags.pack(packer)?;
        self.received_timestamp.pack(packer)?;
        self.solidification_timestamp.pack(packer)?;
        self.branch_id.pack(packer)?;
        self.opinion.pack(packer)
    }

    fn unpack<U: Unpacker, const CHECK: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let flags = Flags::unpack::<_, CHECK>(unpacker).infallible()?;
        let received_timestamp = u64::unpack::<_, CHECK>(unpacker).infallible()?;
        let solidification_timestamp = u64::unpack::<_, CHECK>(unpacker).infallible()?;
        let branch_id = <[u8; 32]>::unpack::<_, CHECK>(unpacker).infallible()?;
        let opinion = Opinion::unpack::<_, CHECK>(unpacker)?;

        Ok(Self {
            flags,
            received_timestamp,
            solidification_timestamp,
            branch_id,
            opinion,
        })
    }
}
