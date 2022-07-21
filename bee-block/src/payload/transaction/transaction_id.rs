// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

use crate::payload::milestone::MilestoneId;

impl_id!(
    pub TransactionId,
    32,
    "A transaction identifier, the BLAKE2b-256 hash of the transaction bytes. See <https://www.blake2.net/> for more information."
);

#[cfg(feature = "serde")]
string_serde_impl!(TransactionId);

impl From<MilestoneId> for TransactionId {
    fn from(milestone_id: MilestoneId) -> Self {
        Self::new(*milestone_id.deref())
    }
}

#[cfg(feature = "inx")]
mod inx {
    use super::*;

    impl From<TransactionId> for inx_bindings::proto::TransactionId {
        fn from(value: TransactionId) -> Self {
            Self { id: value.0.to_vec() }
        }
    }

    impl TryFrom<inx_bindings::proto::TransactionId> for TransactionId {
        type Error = crate::error::inx::InxError;

        fn try_from(value: inx_bindings::proto::TransactionId) -> Result<Self, Self::Error> {
            let bytes: [u8; TransactionId::LENGTH] = value
                .id
                .try_into()
                .map_err(|e| Self::Error::InvalidId("TransactionId", e))?;
            Ok(TransactionId::from(bytes))
        }
    }
}
