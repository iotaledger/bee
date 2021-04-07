// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, MessageId};

use bee_common::packable::{Packable, Read, Write};

use core::{convert::From, ops::Deref, str::FromStr};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TreasuryInput(MessageId);

impl TreasuryInput {
    pub const KIND: u8 = 1;

    pub fn new(id: MessageId) -> Self {
        Self(id)
    }

    pub fn message_id(&self) -> &MessageId {
        &self.0
    }
}

#[cfg(feature = "serde")]
string_serde_impl!(TreasuryInput);

impl Deref for TreasuryInput {
    type Target = MessageId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<MessageId> for TreasuryInput {
    fn from(id: MessageId) -> Self {
        TreasuryInput(id)
    }
}

impl FromStr for TreasuryInput {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TreasuryInput(MessageId::from_str(s)?))
    }
}

impl core::fmt::Display for TreasuryInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::Debug for TreasuryInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "TreasuryInput({})", self.0)
    }
}

impl Packable for TreasuryInput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(MessageId::unpack_inner::<R, CHECK>(reader)?))
    }
}
