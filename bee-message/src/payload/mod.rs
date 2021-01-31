// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod indexation;
pub mod milestone;
pub mod receipt;
pub mod transaction;

use indexation::{IndexationPayload, INDEXATION_PAYLOAD_KIND};
use milestone::{MilestonePayload, MILESTONE_PAYLOAD_KIND};
use receipt::{ReceiptPayload, RECEIPT_PAYLOAD_KIND};
use transaction::{
    TransactionPayload, TreasuryTransactionPayload, TRANSACTION_PAYLOAD_KIND, TREASURY_TRANSACTION_PAYLOAD_KIND,
};

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::boxed::Box;

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Payload {
    Transaction(Box<TransactionPayload>),
    Milestone(Box<MilestonePayload>),
    Indexation(Box<IndexationPayload>),
    Receipt(Box<ReceiptPayload>),
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

impl Packable for Payload {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Transaction(payload) => TRANSACTION_PAYLOAD_KIND.packed_len() + payload.packed_len(),
            Self::Milestone(payload) => MILESTONE_PAYLOAD_KIND.packed_len() + payload.packed_len(),
            Self::Indexation(payload) => INDEXATION_PAYLOAD_KIND.packed_len() + payload.packed_len(),
            Self::Receipt(payload) => RECEIPT_PAYLOAD_KIND.packed_len() + payload.packed_len(),
            Self::TreasuryTransaction(payload) => TREASURY_TRANSACTION_PAYLOAD_KIND.packed_len() + payload.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Transaction(payload) => {
                TRANSACTION_PAYLOAD_KIND.pack(writer)?;
                payload.pack(writer)?;
            }
            Self::Milestone(payload) => {
                MILESTONE_PAYLOAD_KIND.pack(writer)?;
                payload.pack(writer)?;
            }
            Self::Indexation(payload) => {
                INDEXATION_PAYLOAD_KIND.pack(writer)?;
                payload.pack(writer)?;
            }
            Self::Receipt(payload) => {
                RECEIPT_PAYLOAD_KIND.pack(writer)?;
                payload.pack(writer)?;
            }
            Self::TreasuryTransaction(payload) => {
                TREASURY_TRANSACTION_PAYLOAD_KIND.pack(writer)?;
                payload.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u32::unpack(reader)? {
            TRANSACTION_PAYLOAD_KIND => Self::Transaction(Box::new(TransactionPayload::unpack(reader)?)),
            MILESTONE_PAYLOAD_KIND => Self::Milestone(Box::new(MilestonePayload::unpack(reader)?)),
            INDEXATION_PAYLOAD_KIND => Self::Indexation(Box::new(IndexationPayload::unpack(reader)?)),
            RECEIPT_PAYLOAD_KIND => Self::Receipt(Box::new(ReceiptPayload::unpack(reader)?)),
            TREASURY_TRANSACTION_PAYLOAD_KIND => {
                Self::TreasuryTransaction(Box::new(TreasuryTransactionPayload::unpack(reader)?))
            }
            t => return Err(Self::Error::InvalidPayloadKind(t)),
        })
    }
}
