// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;

use bee_message::payload::milestone::MilestoneEssence;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

use std::num::NonZeroU64;

pub mod responses;

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PayloadDto {
    Transaction(TransactionDto),
    Milestone(MilestoneDto),
    Indexation(IndexationDto),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub essence: TransactionEssenceDto,
    #[serde(rename = "unlockBlocks")]
    pub unlock_blocks: Vec<UnlockBlockDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionEssenceDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub inputs: Vec<InputDto>,
    pub outputs: Vec<OutputDto>,
    pub payload: Option<IndexationDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputDto {
    UTXO(UtxoInputDto),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtxoInputDto {
    #[serde(rename = "type")]
    pub kind: u32,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "transactionOutputIndex")]
    pub transaction_output_index: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutputDto {
    SignatureLockedSingle(SignatureLockedSingleOutputDto),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureLockedSingleOutputDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub address: AddressDto,
    pub amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddressDto {
    Ed25519(Ed25519AddressDto),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ed25519AddressDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnlockBlockDto {
    Signature(SignatureUnlockDto),
    Reference(ReferenceUnlockDto),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignatureUnlockDto {
    Ed25519(Ed25519SignatureDto),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ed25519SignatureDto {
    #[serde(rename = "type")]
    pub kind: u32,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferenceUnlockDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilestoneDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub essence: MilestoneEssenceDto,
    pub signatures: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilestoneEssenceDto {
    pub index: u32,
    pub timestamp: u64,
    pub parent_1_message_id: String,
    pub parent_2_message_id: String,
    #[serde(rename = "merkleProof")]
    pub merkle_proof: String,
    #[serde(rename = "publicKeys")]
    pub public_keys: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexationDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: String,
    pub data: String,
}

// &Message -> MessageDto
impl TryFrom<&Message> for MessageDto {
    type Error = &'static str;
    fn try_from(value: &Message) -> Result<Self, Self::Error> {
        Ok(MessageDto {
            network_id: value.network_id().to_string(),
            parent_1_message_id: value.parent1().to_string(),
            parent_2_message_id: value.parent2().to_string(),
            payload: value.payload().as_ref().map(TryInto::try_into).transpose()?,
            nonce: value.nonce().to_string(),
        })
    }
}

// &MessageDto -> Message
impl TryFrom<&MessageDto> for Message {
    type Error = &'static str;
    fn try_from(value: &MessageDto) -> Result<Self, Self::Error> {
        let mut builder = Message::builder()
            .with_network_id(value.network_id.parse::<u64>().map_err(|_| "invalid network id")?)
            .with_parent1(
                value
                    .parent_1_message_id
                    .parse::<MessageId>()
                    .map_err(|_| "invalid parent 1")?,
            )
            .with_parent2(
                value
                    .parent_2_message_id
                    .parse::<MessageId>()
                    .map_err(|_| "invalid parent 2")?,
            )
            .with_nonce(value.nonce.parse::<u64>().map_err(|_| "invalid nonce")?);
        if let Some(p) = value.payload.as_ref() {
            builder = builder.with_payload(p.try_into()?);
        }
        Ok(builder.finish().map_err(|_| "invalid message")?)
    }
}

// &Payload -> PayloadDto
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

// &PayloadDto -> Payload
impl TryFrom<&PayloadDto> for Payload {
    type Error = &'static str;
    fn try_from(value: &PayloadDto) -> Result<Self, Self::Error> {
        match value {
            PayloadDto::Transaction(t) => Ok(Payload::Transaction(t.try_into()?)),
            PayloadDto::Milestone(m) => Ok(Payload::Milestone(m.try_into()?)),
            PayloadDto::Indexation(i) => Ok(Payload::Indexation(i.try_into()?)),
        }
    }
}

// &Box<Transaction> -> TransactionDto
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

// &TransactionDto -> Box<Transaction>
impl TryFrom<&TransactionDto> for Box<Transaction> {
    type Error = &'static str;
    fn try_from(value: &TransactionDto) -> Result<Self, Self::Error> {
        let mut builder = Transaction::builder().with_essence(
            (&value.essence)
                .try_into()
                .map_err(|_| "can not parse transaction essence")?,
        );
        for b in &value.unlock_blocks {
            builder = builder.add_unlock_block(b.try_into().map_err(|_| "can not parse unlock block")?);
        }
        Ok(Box::new(builder.finish().map_err(|_| "can not parse message")?))
    }
}

// &TransactionEssence -> TransactionEssenceDto
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

// &TransactionEssenceDto -> TransactionEssence
impl TryFrom<&TransactionEssenceDto> for TransactionEssence {
    type Error = &'static str;
    fn try_from(value: &TransactionEssenceDto) -> Result<Self, Self::Error> {
        let mut builder = TransactionEssence::builder();

        for i in &value.inputs {
            builder = builder.add_input(i.try_into().map_err(|_| "can not parse input")?);
        }

        for o in &value.outputs {
            builder = builder.add_output(o.try_into().map_err(|_| "can not parse output")?);
        }

        if let Some(p) = &value.payload {
            builder = builder.with_payload(Payload::Indexation((p).try_into()?));
        }

        Ok(builder.finish().map_err(|_| "can not parse transaction essence")?)
    }
}

// &Input -> InputDto
impl TryFrom<&Input> for InputDto {
    type Error = &'static str;
    fn try_from(value: &Input) -> Result<Self, Self::Error> {
        match value {
            Input::UTXO(u) => Ok(InputDto::UTXO(UtxoInputDto {
                kind: 0,
                transaction_id: u.output_id().transaction_id().to_string(),
                transaction_output_index: u.output_id().index(),
            })),
            _ => Err("input type not supported"),
        }
    }
}

// &InputDto -> Input
impl TryFrom<&InputDto> for Input {
    type Error = &'static str;
    fn try_from(value: &InputDto) -> Result<Self, Self::Error> {
        match value {
            InputDto::UTXO(u) => Ok(Input::UTXO(
                UTXOInput::new(
                    u.transaction_id
                        .parse::<TransactionId>()
                        .map_err(|_| "can not parse transaction id")?,
                    u.transaction_output_index,
                )
                .map_err(|_| "can not parse UTXO input")?,
            ))
        }
    }
}

// &Output -> OutputDto
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

// &OutputDto -> Output
impl TryFrom<&OutputDto> for Output {
    type Error = &'static str;
    fn try_from(value: &OutputDto) -> Result<Self, Self::Error> {
        match value {
            OutputDto::SignatureLockedSingle(s) => Ok(Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(
                (&s.address).try_into()?,
                NonZeroU64::new(s.amount.parse::<u64>().map_err(|_| "can not parse amount")?)
                    .ok_or("can not parse amount")?,
            )))
        }
    }
}

// &Address -> AddressDto
impl TryFrom<&Address> for AddressDto {
    type Error = &'static str;
    fn try_from(value: &Address) -> Result<Self, Self::Error> {
        match value {
            Address::Ed25519(ed) => Ok(AddressDto::Ed25519(ed.into())),
            _ => Err("address type not supported"),
        }
    }
}

// &AddressDto -> Address
impl TryFrom<&AddressDto> for Address {
    type Error = &'static str;
    fn try_from(value: &AddressDto) -> Result<Self, Self::Error> {
        match value {
            AddressDto::Ed25519(ed) => Ok(Address::Ed25519(ed.try_into()?)),
        }
    }
}

// &Ed25519Address -> Ed25519AddressDto
impl From<&Ed25519Address> for Ed25519AddressDto {
    fn from(value: &Ed25519Address) -> Self {
        Self {
            kind: 1,
            address: value.to_string(),
        }
    }
}

// &Ed25519AddressDto -> Ed25519Address
impl TryFrom<&Ed25519AddressDto> for Ed25519Address {
    type Error = &'static str;
    fn try_from(value: &Ed25519AddressDto) -> Result<Self, Self::Error> {
        Ok(value
            .address
            .parse::<Ed25519Address>()
            .map_err(|_| "can not parse Ed25519 address")?)
    }
}

// &UnlockBlock -> UnlockBlockDto
impl TryFrom<&UnlockBlock> for UnlockBlockDto {
    type Error = &'static str;
    fn try_from(value: &UnlockBlock) -> Result<Self, Self::Error> {
        match value {
            UnlockBlock::Signature(s) => match s {
                SignatureUnlock::Ed25519(ed) => Ok(UnlockBlockDto::Signature(SignatureUnlockDto::Ed25519(
                    Ed25519SignatureDto {
                        kind: 1,
                        public_key: hex::encode(ed.public_key()),
                        signature: hex::encode(ed.signature()),
                    },
                ))),
                _ => Err("signature unlock type not supported"),
            },
            UnlockBlock::Reference(r) => Ok(UnlockBlockDto::Reference(ReferenceUnlockDto {
                kind: 1,
                index: r.index(),
            })),
            _ => Err("unlock block type not supported"),
        }
    }
}

// &UnlockBlockDto -> UnlockBlock
impl TryFrom<&UnlockBlockDto> for UnlockBlock {
    type Error = &'static str;
    fn try_from(value: &UnlockBlockDto) -> Result<Self, Self::Error> {
        match value {
            UnlockBlockDto::Signature(s) => match s {
                SignatureUnlockDto::Ed25519(ed) => {
                    let mut public_key = [0u8; 32];
                    hex::decode_to_slice(&ed.public_key, &mut public_key).map_err(|_| "invalid public key")?;

                    let signature = hex::decode(&ed.signature)
                        .map_err(|_| "invalid signature")?
                        .into_boxed_slice();

                    Ok(UnlockBlock::Signature(SignatureUnlock::Ed25519(Ed25519Signature::new(
                        public_key, signature,
                    ))))
                }
            },
            UnlockBlockDto::Reference(r) => Ok(UnlockBlock::Reference(
                ReferenceUnlock::new(r.index).map_err(|_| "invalid reference unlock block")?,
            ))
        }
    }
}

// Box<Milestone> -> MilestoneDto
impl From<&Box<Milestone>> for MilestoneDto {
    fn from(value: &Box<Milestone>) -> Self {
        MilestoneDto {
            kind: 1,
            essence: value.essence().into(),
            signatures: value.signatures().iter().map(|s| hex::encode(s)).collect(),
        }
    }
}

// MilestoneDto -> Box<Milestone>
impl TryFrom<&MilestoneDto> for Box<Milestone> {
    type Error = &'static str;
    fn try_from(value: &MilestoneDto) -> Result<Self, Self::Error> {
        let essence = (&value.essence).try_into()?;
        let mut signatures = Vec::new();

        for v in &value.signatures {
            signatures.push(hex::decode(v).map_err(|_| "invalid signature")?.into_boxed_slice())
        }

        Ok(Box::new(Milestone::new(essence, signatures)))
    }
}

// MilestoneEssence -> MilestoneEssenceDto
impl From<&MilestoneEssence> for MilestoneEssenceDto {
    fn from(value: &MilestoneEssence) -> Self {
        MilestoneEssenceDto {
            index: value.index(),
            timestamp: value.timestamp(),
            parent_1_message_id: value.parent1().to_string(),
            parent_2_message_id: value.parent2().to_string(),
            merkle_proof: hex::encode(value.merkle_proof()),
            public_keys: value.public_keys().iter().map(|p| hex::encode(p)).collect(),
        }
    }
}

// MilestoneEssenceDto ->MilestoneEssence
impl TryFrom<&MilestoneEssenceDto> for MilestoneEssence {
    type Error = &'static str;
    fn try_from(value: &MilestoneEssenceDto) -> Result<Self, Self::Error> {
        let index = value.index;
        let timestamp = value.timestamp;
        let parent_1_message_id = value
            .parent_1_message_id
            .parse::<MessageId>()
            .map_err(|_| "invalid parent1 in milestone payload")?;
        let parent_2_message_id = value
            .parent_2_message_id
            .parse::<MessageId>()
            .map_err(|_| "invalid parent2 in milestone payload")?;
        let merkle_proof = hex::decode(&value.merkle_proof)
            .map_err(|_| "invalid merkle proof")?
            .into_boxed_slice();
        let mut public_keys = Vec::new();

        for v in &value.public_keys {
            let mut p = [0u8; 32];
            hex::decode_to_slice(v, &mut p).map_err(|_| "invalid public key")?;
            public_keys.push(p);
        }

        Ok(MilestoneEssence::new(
            index,
            timestamp,
            parent_1_message_id,
            parent_2_message_id,
            merkle_proof,
            public_keys,
        ))
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

// IndexationDto -> Box<Indexation>
impl TryFrom<&IndexationDto> for Box<Indexation> {
    type Error = &'static str;
    fn try_from(value: &IndexationDto) -> Result<Self, Self::Error> {
        Ok(Box::new(
            Indexation::new(
                value.index.clone(),
                &hex::decode(value.data.clone()).map_err(|_| "invalid data in indexation")?,
            )
            .map_err(|_| "invalid indexation")?,
        ))
    }
}
