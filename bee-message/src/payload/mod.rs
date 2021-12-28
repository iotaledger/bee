// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The payload module defines the core data types for representing message payloads.

pub mod indexation;
pub mod milestone;
pub mod receipt;
pub mod transaction;
pub mod treasury;

pub use indexation::IndexationPayload;
pub(crate) use indexation::{IndexationDataLength, IndexationIndexLength};
pub use milestone::MilestonePayload;
pub(crate) use milestone::{PublicKeyCount, SignatureCount};
pub(crate) use receipt::ReceiptFundsCount;
pub use receipt::ReceiptPayload;
pub use transaction::TransactionPayload;
pub(crate) use transaction::{InputCount, OutputCount};
pub use treasury::TreasuryTransactionPayload;

use crate::Error;

use bee_common::packable::{Read, Write};
use bee_packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
};

use alloc::boxed::Box;
use core::ops::Deref;

/// A generic payload that can represent different types defining message payloads.
#[derive(Clone, Debug, Eq, PartialEq, bee_packable::Packable)]
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
    /// An indexation payload.
    #[packable(tag = IndexationPayload::KIND)]
    Indexation(Box<IndexationPayload>),
    /// A receipt payload.
    #[packable(tag = ReceiptPayload::KIND)]
    Receipt(Box<ReceiptPayload>),
    /// A treasury transaction payload.
    #[packable(tag = TreasuryTransactionPayload::KIND)]
    TreasuryTransaction(Box<TreasuryTransactionPayload>),
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

impl From<IndexationPayload> for Payload {
    fn from(payload: IndexationPayload) -> Self {
        Self::Indexation(Box::new(payload))
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

impl Payload {
    /// Returns the payload kind of a `Payload`.
    pub fn kind(&self) -> u32 {
        match self {
            Self::Transaction(_) => TransactionPayload::KIND,
            Self::Milestone(_) => MilestonePayload::KIND,
            Self::Indexation(_) => IndexationPayload::KIND,
            Self::Receipt(_) => ReceiptPayload::KIND,
            Self::TreasuryTransaction(_) => TreasuryTransactionPayload::KIND,
        }
    }
}

impl bee_common::packable::Packable for Payload {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Transaction(payload) => TransactionPayload::KIND.packed_len() + payload.packed_len(),
            Self::Milestone(payload) => MilestonePayload::KIND.packed_len() + payload.packed_len(),
            Self::Indexation(payload) => IndexationPayload::KIND.packed_len() + payload.packed_len(),
            Self::Receipt(payload) => ReceiptPayload::KIND.packed_len() + payload.packed_len(),
            Self::TreasuryTransaction(payload) => TreasuryTransactionPayload::KIND.packed_len() + payload.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Transaction(payload) => {
                TransactionPayload::KIND.pack(writer)?;
                payload.pack(writer)?;
            }
            Self::Milestone(payload) => {
                MilestonePayload::KIND.pack(writer)?;
                payload.pack(writer)?;
            }
            Self::Indexation(payload) => {
                IndexationPayload::KIND.pack(writer)?;
                payload.pack(writer)?;
            }
            Self::Receipt(payload) => {
                ReceiptPayload::KIND.pack(writer)?;
                payload.pack(writer)?;
            }
            Self::TreasuryTransaction(payload) => {
                TreasuryTransactionPayload::KIND.pack(writer)?;
                payload.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u32::unpack_inner::<R, CHECK>(reader)? {
            TransactionPayload::KIND => TransactionPayload::unpack_inner::<R, CHECK>(reader)?.into(),
            MilestonePayload::KIND => MilestonePayload::unpack_inner::<R, CHECK>(reader)?.into(),
            IndexationPayload::KIND => IndexationPayload::unpack_inner::<R, CHECK>(reader)?.into(),
            ReceiptPayload::KIND => ReceiptPayload::unpack_inner::<R, CHECK>(reader)?.into(),
            TreasuryTransactionPayload::KIND => TreasuryTransactionPayload::unpack_inner::<R, CHECK>(reader)?.into(),
            k => return Err(Self::Error::InvalidPayloadKind(k)),
        })
    }
}

/// Returns the packed length of an optional payload.
pub fn option_payload_packed_len(payload: Option<&Payload>) -> usize {
    use bee_common::packable::Packable;

    0u32.packed_len() + payload.map_or(0, Packable::packed_len)
}

/// Packs an optional payload to a writer.
pub fn option_payload_pack<W: Write>(writer: &mut W, payload: Option<&Payload>) -> Result<(), Error> {
    use bee_common::packable::Packable;

    if let Some(payload) = payload {
        (payload.packed_len() as u32).pack(writer)?;
        payload.pack(writer)?;
    } else {
        0u32.pack(writer)?;
    }

    Ok(())
}

/// Unpacks an optional payload from a reader.
pub fn option_payload_unpack<R: Read + ?Sized, const CHECK: bool>(
    reader: &mut R,
) -> Result<(usize, Option<Payload>), Error> {
    use bee_common::packable::Packable;

    let payload_len = u32::unpack_inner::<R, CHECK>(reader)? as usize;

    if payload_len > 0 {
        let payload = Payload::unpack_inner::<R, CHECK>(reader)?;
        if payload_len != payload.packed_len() {
            Err(Error::InvalidPayloadLength {
                expected: payload_len,
                actual: payload.packed_len(),
            })
        } else {
            Ok((payload_len, Some(payload)))
        }
    } else {
        Ok((0, None))
    }
}

/// Representation of an optional [`Payload`].
/// Essentially an `Option<Payload>` with a different [`Packable`] implementation, to conform to specs.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct OptionalPayload(Option<Payload>);

impl OptionalPayload {
    fn pack_ref<P: Packer>(payload: &Payload, packer: &mut P) -> Result<(), P::Error> {
        use bee_packable::{Packable, PackableExt};

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

impl bee_packable::Packable for OptionalPayload {
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
        use bee_packable::PackableExt;
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
