// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::signature::Signature;

use bee_packable::Packable;

/// An [`UnlockBlock`](crate::unlock::UnlockBlock) which is used to unlock a signature locked
/// [`Input`](crate::input::Input).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Packable)]
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
