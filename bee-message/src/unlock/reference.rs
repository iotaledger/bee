// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{constants::INPUT_OUTPUT_INDEX_RANGE, Error};

use bee_common::packable::{Packable, Read, Write};

use core::convert::TryFrom;

/// A `ReferenceUnlock` is an [`UnlockBlock`](crate::unlock::UnlockBlock) that refers to a
/// [`SignatureUnlock`](crate::unlock::SignatureUnlock).
///
/// It consists of an index to a previous `UnlockBlock` which **must** be a `SignatureUnlock`.
/// Referring to another `ReferenceUnlock` is invalid and will be rejected by the node's protocol
/// validation.
///
/// Spec: #iota-protocol-rfc-draft
/// <https://github.com/luca-moser/protocol-rfcs/blob/signed-tx-payload/text/0000-transaction-payload/0000-transaction-payload.md#reference-unlock-block>
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReferenceUnlock(u16);

impl ReferenceUnlock {
    /// The kind of UnlockBlock: `1` as defined by the protocol.
    pub const KIND: u8 = 1;

    /// Create a new `ReferenceUnlock` from an index.
    ///
    /// The index must within the range of valid indices as defined by the protocol, and must refer
    /// to a *previous* unlock block (not a later one).
    ///
    /// The caller is responsible for validating that the referenced block is a previous
    /// `SignatureUnlock` block.
    pub fn new(index: u16) -> Result<Self, Error> {
        if !INPUT_OUTPUT_INDEX_RANGE.contains(&index) {
            return Err(Error::InvalidReferenceIndex(index));
        }

        Ok(Self(index))
    }

    /// Return the underlying UnlockBlock index that this `ReferenceUnlock` refers to.
    pub fn index(&self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for ReferenceUnlock {
    type Error = Error;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        Self::new(index)
    }
}

impl Packable for ReferenceUnlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u16.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Self::new(u16::unpack_inner::<R, CHECK>(reader)?)
    }
}
