// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{signature::Signature, Error};

use bee_common::packable::{Packable, Read, Write};

/// An [`UnlockBlock`](crate::unlock_block::UnlockBlock) which is used to unlock a signature locked
/// [`Input`](crate::input::Input).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, derive_more::Deref)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize, derive_more::From))]
pub struct SignatureUnlockBlock(Signature);

impl SignatureUnlockBlock {
    /// The [`UnlockBlock`](crate::unlock_block::UnlockBlock) kind of a [`SignatureUnlockBlock`].
    pub const KIND: u8 = 0;

    /// Creates a new [`SignatureUnlockBlock`].
    #[inline(always)]
    pub fn new(signature: Signature) -> Self {
        Self(signature)
    }

    /// Returns the actual [`Signature`] of the [`SignatureUnlockBlock`].
    #[inline(always)]
    pub fn signature(&self) -> &Signature {
        &self.0
    }
}

impl Packable for SignatureUnlockBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(Signature::unpack_inner::<R, CHECK>(reader)?))
    }
}
