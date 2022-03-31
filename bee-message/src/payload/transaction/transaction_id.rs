// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

use crate::payload::milestone::MilestoneId;

impl_id!(
    pub TransactionId,
    32,
    "A transaction identifier, the BLAKE2b-256 hash of the transaction bytes. See <https://www.blake2.net/> for more information."
);

#[cfg(feature = "serde1")]
string_serde_impl!(TransactionId);

impl From<MilestoneId> for TransactionId {
    fn from(milestone_id: MilestoneId) -> Self {
        Self::new(*milestone_id.deref())
    }
}
