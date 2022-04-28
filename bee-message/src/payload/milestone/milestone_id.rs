// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

impl_id!(
    pub MilestoneId,
    32,
    "A milestone identifier, the BLAKE2b-256 hash of the milestone bytes. See <https://www.blake2.net/> for more information."
);

impl MilestoneId {
    /// Create a null [`MilestoneId`].
    pub fn null() -> Self {
        Self([0u8; MilestoneId::LENGTH])
    }
}

#[cfg(feature = "serde")]
string_serde_impl!(MilestoneId);
