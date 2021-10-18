// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The payload module defines the core data types for representing message payloads.

pub mod indexation;
pub mod milestone;
pub mod receipt;
pub mod transaction;
pub mod treasury;

use indexation::IndexationPayload;
use milestone::MilestonePayload;
use receipt::ReceiptPayload;
use transaction::TransactionPayload;
use treasury::TreasuryTransactionPayload;

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use alloc::boxed::Box;

/// A generic payload that can represent different types defining message payloads.
#[derive(Clone, Debug, Eq, PartialEq, derive_more::From)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Payload {
    /// A transaction payload.
    Transaction(Box<TransactionPayload>),
    /// A milestone payload.
    Milestone(Box<MilestonePayload>),
    /// An indexation payload.
    Indexation(Box<IndexationPayload>),
    /// A receipt payload.
    Receipt(Box<ReceiptPayload>),
    /// A treasury transaction payload.
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

impl Packable for Payload {
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
            TransactionPayload::KIND => Box::new(TransactionPayload::unpack_inner::<R, CHECK>(reader)?).into(),
            MilestonePayload::KIND => Box::new(MilestonePayload::unpack_inner::<R, CHECK>(reader)?).into(),
            IndexationPayload::KIND => Box::new(IndexationPayload::unpack_inner::<R, CHECK>(reader)?).into(),
            ReceiptPayload::KIND => Box::new(ReceiptPayload::unpack_inner::<R, CHECK>(reader)?).into(),
            TreasuryTransactionPayload::KIND => {
                Box::new(TreasuryTransactionPayload::unpack_inner::<R, CHECK>(reader)?).into()
            }
            k => return Err(Self::Error::InvalidPayloadKind(k)),
        })
    }
}

/// Returns the packed length of an optional payload.
pub fn option_payload_packed_len(payload: Option<&Payload>) -> usize {
    0u32.packed_len() + payload.map_or(0, Packable::packed_len)
}

/// Packs an optional payload to a writer.
pub fn option_payload_pack<W: Write>(writer: &mut W, payload: Option<&Payload>) -> Result<(), Error> {
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
    let payload_len = u32::unpack_inner::<R, CHECK>(reader)? as usize;

    if payload_len > 0 {
        let payload = Payload::unpack_inner::<R, CHECK>(reader)?;
        if payload_len != payload.packed_len() {
            Err(Error::InvalidPayloadLength(payload_len, payload.packed_len()))
        } else {
            Ok((payload_len, Some(payload)))
        }
    } else {
        Ok((0, None))
    }
}
