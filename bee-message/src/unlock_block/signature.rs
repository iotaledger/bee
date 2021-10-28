// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{signature::Signature, Error};

use bee_common::packable::{Packable, Read, Write};

/// An [`UnlockBlock`](crate::unlock::UnlockBlock) which is used to unlock a signature locked
/// [`Input`](crate::input::Input).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct SignatureUnlock(Signature);

impl SignatureUnlock {
    /// The [`UnlockBlock`](crate::unlock::UnlockBlock) kind of a [`SignatureUnlock`].
    pub const KIND: u8 = 0;

    /// Creates a new [`SignatureUnlock`].
    pub fn new(signature: Signature) -> Self {
        Self(signature)
    }

    /// Returns the actual [`Signature`] of the [`SignatureUnlock`].
    pub fn signature(&self) -> &Signature {
        &self.0
    }
}

impl From<Signature> for SignatureUnlock {
    fn from(signature: Signature) -> Self {
        Self::new(signature)
    }
}

impl core::ops::Deref for SignatureUnlock {
    type Target = Signature;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Packable for SignatureUnlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(Signature::unpack_inner::<R, CHECK>(reader)?))
    }
}