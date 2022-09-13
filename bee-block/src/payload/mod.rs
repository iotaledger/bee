// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The payload module defines the core data types for representing block payloads.

pub mod milestone;
pub mod tagged_data;
pub mod transaction;
pub mod treasury_transaction;

use alloc::boxed::Box;
use core::ops::Deref;

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
};

pub(crate) use self::{
    milestone::{MilestoneMetadataLength, MilestoneOptionCount, ReceiptFundsCount, SignatureCount},
    tagged_data::{TagLength, TaggedDataLength},
    transaction::{InputCount, OutputCount},
};
pub use self::{
    milestone::{MilestoneOptions, MilestonePayload},
    tagged_data::TaggedDataPayload,
    transaction::TransactionPayload,
    treasury_transaction::TreasuryTransactionPayload,
};
use crate::{protocol::ProtocolParameters, Error};

/// A generic payload that can represent different types defining block payloads.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Payload {
    /// A transaction payload.
    Transaction(Box<TransactionPayload>),
    /// A milestone payload.
    Milestone(Box<MilestonePayload>),
    /// A treasury transaction payload.
    TreasuryTransaction(Box<TreasuryTransactionPayload>),
    /// A tagged data payload.
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
            Self::TreasuryTransaction(_) => TreasuryTransactionPayload::KIND,
            Self::TaggedData(_) => TaggedDataPayload::KIND,
        }
    }
}

impl Packable for Payload {
    type UnpackError = Error;
    type UnpackVisitor = ProtocolParameters;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match self {
            Payload::Transaction(transaction) => {
                TransactionPayload::KIND.pack(packer)?;
                transaction.pack(packer)
            }
            Payload::Milestone(milestone) => {
                MilestonePayload::KIND.pack(packer)?;
                milestone.pack(packer)
            }
            Payload::TreasuryTransaction(treasury_transaction) => {
                TreasuryTransactionPayload::KIND.pack(packer)?;
                treasury_transaction.pack(packer)
            }
            Payload::TaggedData(tagged_data) => {
                TaggedDataPayload::KIND.pack(packer)?;
                tagged_data.pack(packer)
            }
        }?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
        visitor: &Self::UnpackVisitor,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        Ok(match u32::unpack::<_, VERIFY>(unpacker, &()).coerce()? {
            TransactionPayload::KIND => {
                Payload::from(TransactionPayload::unpack::<_, VERIFY>(unpacker, visitor).coerce()?)
            }
            MilestonePayload::KIND => Payload::from(MilestonePayload::unpack::<_, VERIFY>(unpacker, visitor).coerce()?),
            TreasuryTransactionPayload::KIND => {
                Payload::from(TreasuryTransactionPayload::unpack::<_, VERIFY>(unpacker, visitor).coerce()?)
            }
            TaggedDataPayload::KIND => Payload::from(TaggedDataPayload::unpack::<_, VERIFY>(unpacker, &()).coerce()?),
            k => return Err(Error::InvalidPayloadKind(k)).map_err(UnpackError::Packable),
        })
    }
}

/// Representation of an optional [`Payload`].
/// Essentially an `Option<Payload>` with a different [`Packable`] implementation, to conform to specs.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    type UnpackVisitor = ProtocolParameters;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match &self.0 {
            None => 0u32.pack(packer),
            Some(payload) => Self::pack_ref(payload, packer),
        }
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
        visitor: &Self::UnpackVisitor,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let len = u32::unpack::<_, VERIFY>(unpacker, &()).coerce()? as usize;

        if len > 0 {
            unpacker.ensure_bytes(len)?;

            let start_opt = unpacker.read_bytes();

            let payload = Payload::unpack::<_, VERIFY>(unpacker, visitor)?;

            let actual_len = if let (Some(start), Some(end)) = (start_opt, unpacker.read_bytes()) {
                end - start
            } else {
                payload.packed_len()
            };

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

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    pub use super::{
        milestone::dto::{try_from_milestone_payload_dto_for_milestone_payload, MilestonePayloadDto},
        tagged_data::dto::TaggedDataPayloadDto,
        transaction::dto::{try_from_transaction_payload_dto_for_transaction_payload, TransactionPayloadDto},
        treasury_transaction::dto::{
            try_from_treasury_transaction_payload_dto_for_treasury_transaction_payload, TreasuryTransactionPayloadDto,
        },
    };
    use crate::error::dto::DtoError;

    /// Describes all the different payload types.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum PayloadDto {
        Transaction(Box<TransactionPayloadDto>),
        Milestone(Box<MilestonePayloadDto>),
        TreasuryTransaction(Box<TreasuryTransactionPayloadDto>),
        TaggedData(Box<TaggedDataPayloadDto>),
    }

    impl From<&Payload> for PayloadDto {
        fn from(value: &Payload) -> Self {
            match value {
                Payload::Transaction(p) => PayloadDto::Transaction(Box::new(TransactionPayloadDto::from(p.as_ref()))),
                Payload::Milestone(p) => PayloadDto::Milestone(Box::new(MilestonePayloadDto::from(p.as_ref()))),
                Payload::TreasuryTransaction(p) => {
                    PayloadDto::TreasuryTransaction(Box::new(TreasuryTransactionPayloadDto::from(p.as_ref())))
                }
                Payload::TaggedData(p) => PayloadDto::TaggedData(Box::new(TaggedDataPayloadDto::from(p.as_ref()))),
            }
        }
    }

    pub fn try_from_payload_dto_payload(
        value: &PayloadDto,
        protocol_parameters: &ProtocolParameters,
    ) -> Result<Payload, DtoError> {
        Ok(match value {
            PayloadDto::Transaction(p) => Payload::from(try_from_transaction_payload_dto_for_transaction_payload(
                p.as_ref(),
                protocol_parameters,
            )?),
            PayloadDto::Milestone(p) => Payload::from(try_from_milestone_payload_dto_for_milestone_payload(
                p.as_ref(),
                protocol_parameters,
            )?),
            PayloadDto::TreasuryTransaction(p) => Payload::from(
                try_from_treasury_transaction_payload_dto_for_treasury_transaction_payload(
                    p.as_ref(),
                    protocol_parameters,
                )?,
            ),
            PayloadDto::TaggedData(p) => Payload::from(TaggedDataPayload::try_from(p.as_ref())?),
        })
    }
}
