// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use derive_more::{Deref, From};

use crate::signature::Signature;

/// An [`UnlockBlock`](crate::unlock_block::UnlockBlock) which is used to unlock a signature locked
/// [`Input`](crate::input::Input).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, From, Deref, packable::Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use crate::signature::dto::SignatureDto;

    /// Defines an unlock block containing signature(s) unlocking input(s).
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct SignatureUnlockBlockDto {
        #[serde(rename = "type")]
        pub kind: u8,
        pub signature: SignatureDto,
    }
}
