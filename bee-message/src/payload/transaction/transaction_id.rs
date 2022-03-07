// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::payload::milestone::MilestoneId;

use core::ops::Deref;

impl_id!(
    pub TransactionId,
    32,
    "A transaction identifier, the BLAKE2b-256 hash of the transaction bytes. See <https://www.blake2.net/> for more information."
);

#[cfg(feature = "serde1")]
string_serde_impl!(TransactionId);

impl From<MilestoneId> for TransactionId {
    fn from(milestone_id: MilestoneId) -> Self {
        // SAFETY: lengths are known to be the same.
        Self::new(*milestone_id.deref())
    }
}
