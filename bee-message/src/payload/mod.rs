// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The payload module defines the core data types for representing message payloads.

pub mod milestone;
pub mod receipt;
pub mod tagged_data;
pub mod transaction;
pub mod treasury_transaction;

pub use milestone::MilestonePayload;
pub(crate) use milestone::{PublicKeyCount, SignatureCount};
pub use receipt::ReceiptPayload;
pub(crate) use receipt::{MigratedFundsAmount, ReceiptFundsCount};
pub use tagged_data::TaggedDataPayload;
pub(crate) use tagged_data::{TagLength, TaggedDataLength};
pub use transaction::TransactionPayload;
pub(crate) use transaction::{InputCount, OutputCount};
pub use treasury_transaction::TreasuryTransactionPayload;

use crate::Error;

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
};

use alloc::boxed::Box;
use core::ops::Deref;

/// A generic payload that can represent different types defining message payloads.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u32, with_error = Error::InvalidPayloadKind)]
pub enum Payload {
    /// A transaction payload.
    #[packable(tag = TransactionPayload::KIND)]
    Transaction(Box<TransactionPayload>),
    /// A milestone payload.
    #[packable(tag = MilestonePayload::KIND)]
    Milestone(Box<MilestonePayload>),
    /// A receipt payload.
    #[packable(tag = ReceiptPayload::KIND)]
    Receipt(Box<ReceiptPayload>),
    /// A treasury transaction payload.
    #[packable(tag = TreasuryTransactionPayload::KIND)]
    TreasuryTransaction(Box<TreasuryTransactionPayload>),
    /// A tagged data payload.
    #[packable(tag = TaggedDataPayload::KIND)]
    TaggedData(Box<TaggedDataPayload>),
}

impl From<TransactionPayload> for Payload {
    fn from(payload: TransactionPayload) -> Self {
        Self::Transaction(Box::new(payload))
    }
}

impl From<MilestonePayload> for Payload {
    fn from(payload: MilestonePayload) -> Self {
        Self::Milestone(Box::new(payload))
    }
}

impl From<ReceiptPayload> for Payload {
    fn from(payload: ReceiptPayload) -> Self {
        Self::Receipt(Box::new(payload))
    }
}

impl From<TreasuryTransactionPayload> for Payload {
    fn from(payload: TreasuryTransactionPayload) -> Self {
        Self::TreasuryTransaction(Box::new(payload))
    }
}

impl From<TaggedDataPayload> for Payload {
    fn from(payload: TaggedDataPayload) -> Self {
        Self::TaggedData(Box::new(payload))
    }
}

impl Payload {
    /// Returns the payload kind of a `Payload`.
    pub fn kind(&self) -> u32 {
        match self {
            Self::Transaction(_) => TransactionPayload::KIND,
            Self::Milestone(_) => MilestonePayload::KIND,
            Self::Receipt(_) => ReceiptPayload::KIND,
            Self::TreasuryTransaction(_) => TreasuryTransactionPayload::KIND,
            Self::TaggedData(_) => TaggedDataPayload::KIND,
        }
    }
}

/// Representation of an optional [`Payload`].
/// Essentially an `Option<Payload>` with a different [`Packable`] implementation, to conform to specs.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct OptionalPayload(Option<Payload>);

impl OptionalPayload {
    fn pack_ref<P: Packer>(payload: &Payload, packer: &mut P) -> Result<(), P::Error> {
        (payload.packed_len() as u32).pack(packer)?;
        payload.pack(packer)
    }
}

impl Deref for OptionalPayload {
    type Target = Option<Payload>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Packable for OptionalPayload {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match &self.0 {
            None => 0u32.pack(packer),
            Some(payload) => Self::pack_ref(payload, packer),
        }
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let len = u32::unpack::<_, VERIFY>(unpacker).infallible()? as usize;

        if len > 0 {
            unpacker.ensure_bytes(len)?;

            let payload = Payload::unpack::<_, VERIFY>(unpacker)?;
            let actual_len = payload.packed_len();

            if len != actual_len {
                Err(UnpackError::Packable(Error::InvalidPayloadLength {
                    expected: len,
                    actual: actual_len,
                }))
            } else {
                Ok(Self(Some(payload)))
            }
        } else {
            Ok(Self(None))
        }
    }
}

// FIXME: does this break any invariant about the Payload length?
impl From<Option<Payload>> for OptionalPayload {
    fn from(option: Option<Payload>) -> Self {
        Self(option)
    }
}

#[allow(clippy::from_over_into)]
impl Into<Option<Payload>> for OptionalPayload {
    fn into(self) -> Option<Payload> {
        self.0
    }
}
