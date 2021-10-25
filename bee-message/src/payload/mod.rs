// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The payload module defines the core data types for representing message payloads.

pub mod indexation;
pub mod milestone;
pub mod receipt;
pub mod transaction;
pub mod treasury;

use std::convert::Infallible;

use indexation::IndexationPayload;
use milestone::MilestonePayload;
use receipt::ReceiptPayload;
use transaction::TransactionPayload;
use treasury::TreasuryTransactionPayload;

use crate::Error;

use bee_packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
};

use alloc::boxed::Box;

/// A generic payload that can represent different types defining message payloads.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u32, with_error = Error::InvalidPayloadKind)]
#[packable(unpack_error = Error)]
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

/// Packs an optional payload to a packer.
pub fn option_payload_pack<P: Packer>(packer: &mut P, payload: Option<&Payload>) -> Result<(), P::Error> {
    if let Some(payload) = payload {
        (payload.packed_len() as u32).pack(packer)?;
        payload.pack(packer)?;
    } else {
        0u32.pack(packer)?;
    }

    Ok(())
}

/// Unpacks an optional payload from an unpacker.
pub fn option_payload_unpack<U: Unpacker, const VERIFY: bool>(
    unpacker: &mut U,
) -> Result<(usize, Option<Payload>), UnpackError<Error, U::Error>> {
    let payload_len = u32::unpack::<_, VERIFY>(unpacker).infallible()? as usize;

    if payload_len > 0 {
        let payload = Payload::unpack::<_, VERIFY>(unpacker)?;
        if payload_len != payload.packed_len() {
            Err(UnpackError::Packable(Error::InvalidPayloadLength(
                payload_len,
                payload.packed_len(),
            )))
        } else {
            Ok((payload_len, Some(payload)))
        }
    } else {
        Ok((0, None))
    }
}
