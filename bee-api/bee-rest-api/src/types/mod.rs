// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;

use serde::Serialize;
use std::convert::{TryFrom, TryInto};

pub mod requests;
pub mod responses;

#[derive(Clone, Debug, Serialize)]
pub struct MessageDto {
    #[serde(rename = "networkId")]
    pub network_id: String,
    #[serde(rename = "parent1MessageId")]
    pub parent_1_message_id: String,
    #[serde(rename = "parent2MessageId")]
    pub parent_2_message_id: String,
    pub payload: Option<PayloadDto>,
    pub nonce: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum PayloadDto {
    Transaction(TransactionDto),
    Milestone(MilestoneDto),
    Indexation(IndexationDto),
}

#[derive(Clone, Debug, Serialize)]
pub struct TransactionDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub essence: TransactionEssenceDto,
    #[serde(rename = "unlockBlocks")]
    pub unlock_blocks: Vec<UnlockBlockDto>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MilestoneDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: u32,
    pub timestamp: u64,
    #[serde(rename = "inclusionMerkleProof")]
    pub inclusion_merkle_proof: String,
    pub signatures: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct IndexationDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: String,
    pub data: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct TransactionEssenceDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub inputs: Vec<InputDto>,
    pub outputs: Vec<OutputDto>,
    pub payload: Option<IndexationDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum InputDto {
    Utxo(UtxoInputDto),
}

#[derive(Clone, Debug, Serialize)]
pub struct UtxoInputDto {
    #[serde(rename = "type")]
    pub kind: u32,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "transactionOutputIndex")]
    pub transaction_output_index: u16,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum OutputDto {
    SignatureLockedSingle(SignatureLockedSingleOutputDto),
}

#[derive(Clone, Debug, Serialize)]
pub struct SignatureLockedSingleOutputDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub address: AddressDto,
    pub amount: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum AddressDto {
    Ed25519(Ed25519AddressDto),
}

#[derive(Clone, Debug, Serialize)]
pub struct Ed25519AddressDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub address: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum UnlockBlockDto {
    Signature(SignatureUnlockDto),
    Reference(ReferenceUnlockDto),
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum SignatureUnlockDto {
    Ed25519(Ed25519SignatureDto),
}

#[derive(Clone, Debug, Serialize)]
pub struct Ed25519SignatureDto {
    #[serde(rename = "type")]
    pub kind: u32,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReferenceUnlockDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub reference: u16,
}

// Message -> MessageDto
impl TryFrom<&Message> for MessageDto {
    type Error = &'static str;
    fn try_from(value: &Message) -> Result<Self, Self::Error> {
        Ok(MessageDto {
            network_id: value.network_id().to_string(),
            parent_1_message_id: value.parent1().to_string(),
            parent_2_message_id: value.parent2().to_string(),
            payload: match value.payload() {
                Some(p) => Some(p.try_into()?),
                None => None,
            },
            nonce: value.nonce().to_string(),
        })
    }
}

impl TryFrom<&Payload> for PayloadDto {
    type Error = &'static str;
    fn try_from(value: &Payload) -> Result<Self, Self::Error> {
        match value {
            Payload::Transaction(t) => Ok(PayloadDto::Transaction(t.try_into()?)),
            Payload::Milestone(m) => Ok(PayloadDto::Milestone(m.into())),
            Payload::Indexation(i) => Ok(PayloadDto::Indexation(i.into())),
            _ => Err("payload type not supported"),
        }
    }
}

impl TryFrom<&Box<Transaction>> for TransactionDto {
    type Error = &'static str;
    fn try_from(value: &Box<Transaction>) -> Result<Self, Self::Error> {
        Ok(TransactionDto {
            kind: 0,
            essence: value.essence().try_into()?,
            unlock_blocks: value
                .unlock_blocks()
                .iter()
                .map(|u| u.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<&Box<Milestone>> for MilestoneDto {
    fn from(value: &Box<Milestone>) -> Self {
        MilestoneDto {
            kind: 1,
            index: value.essence().index(),
            timestamp: value.essence().timestamp(),
            inclusion_merkle_proof: hex::encode(value.essence().merkle_proof()),
            signatures: value.signatures().iter().map(|s| hex::encode(s)).collect(),
        }
    }
}

impl From<&Box<Indexation>> for IndexationDto {
    fn from(value: &Box<Indexation>) -> Self {
        IndexationDto {
            kind: 2,
            index: value.index().to_owned(),
            data: hex::encode(value.data()),
        }
    }
}

impl TryFrom<&TransactionEssence> for TransactionEssenceDto {
    type Error = &'static str;
    fn try_from(value: &TransactionEssence) -> Result<Self, Self::Error> {
        Ok(TransactionEssenceDto {
            kind: 0, // TODO: should this be removed?
            inputs: value
                .inputs()
                .iter()
                .map(|i| i.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            outputs: value
                .outputs()
                .iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            payload: match value.payload() {
                Some(Payload::Indexation(i)) => Some(i.into()),
                Some(_) => return Err("a transaction payload only can have nested indexation payload"),
                None => None,
            },
        })
    }
}

impl TryFrom<&Input> for InputDto {
    type Error = &'static str;
    fn try_from(value: &Input) -> Result<Self, Self::Error> {
        match value {
            Input::UTXO(u) => Ok(InputDto::Utxo(UtxoInputDto {
                kind: 0,
                transaction_id: u.output_id().transaction_id().to_string(),
                transaction_output_index: u.output_id().index(),
            })),
            _ => Err("input type not supported"),
        }
    }
}

impl TryFrom<&Output> for OutputDto {
    type Error = &'static str;
    fn try_from(value: &Output) -> Result<Self, Self::Error> {
        match value {
            Output::SignatureLockedSingle(s) => Ok(OutputDto::SignatureLockedSingle(SignatureLockedSingleOutputDto {
                kind: 0,
                address: s.address().try_into()?,
                amount: s.amount().to_string(),
            })),
            _ => return Err("output type not supported"),
        }
    }
}

impl TryFrom<&Address> for AddressDto {
    type Error = &'static str;
    fn try_from(value: &Address) -> Result<Self, Self::Error> {
        match value {
            Address::Ed25519(ed) => Ok(AddressDto::Ed25519(ed.into())),
            _ => Err("address type not supported"),
        }
    }
}

impl From<&Ed25519Address> for Ed25519AddressDto {
    fn from(value: &Ed25519Address) -> Self {
        Self {
            kind: 0,
            address: value.to_string(),
        }
    }
}

impl TryFrom<&UnlockBlock> for UnlockBlockDto {
    type Error = &'static str;
    fn try_from(value: &UnlockBlock) -> Result<Self, Self::Error> {
        match value {
            UnlockBlock::Signature(s) => match s {
                SignatureUnlock::Ed25519(ed) => Ok(UnlockBlockDto::Signature(SignatureUnlockDto::Ed25519(
                    Ed25519SignatureDto {
                        kind: 0,
                        public_key: hex::encode(ed.public_key()),
                        signature: hex::encode(ed.signature()),
                    },
                ))),
                _ => Err("signature unlock type not supported"),
            },
            UnlockBlock::Reference(r) => Ok(UnlockBlockDto::Reference(ReferenceUnlockDto {
                kind: 0,
                reference: r.index(),
            })),
            _ => Err("unlock block type not supported"),
        }
    }
}
