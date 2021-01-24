// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_pow::providers::{ConstantBuilder, ProviderBuilder};

use serde::{Deserialize, Serialize};

use std::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerDto {
    pub id: String,
    pub alias: String,
    #[serde(rename = "multiAddresses")]
    pub multi_addresses: Vec<String>,
    pub relation: String,
    pub connected: bool,
    #[serde(rename = "gossipMetrics")]
    pub gossip_metrics: GossipMetricsDto,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GossipMetricsDto {
    #[serde(rename = "receivedMessages")]
    pub received_messages: u64,
    #[serde(rename = "newMessages")]
    pub new_messages: u64,
    #[serde(rename = "knownMessages")]
    pub known_messages: u64,
    #[serde(rename = "receivedMessageRequests")]
    pub received_message_requests: u64,
    #[serde(rename = "receivedMilestoneRequests")]
    pub received_milestone_requests: u64,
    #[serde(rename = "receivedHeartbeats")]
    pub received_heartbeats: u64,
    #[serde(rename = "sentMessages")]
    pub sent_messages: u64,
    #[serde(rename = "sentMessageRequests")]
    pub sent_message_requests: u64,
    #[serde(rename = "sentMilestoneRequests")]
    pub sent_milestone_requests: u64,
    #[serde(rename = "sentHeartbeats")]
    pub sent_heartbeats: u64,
    #[serde(rename = "droppedPackets")]
    pub dropped_packets: u64,
}

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
    pub amount: u64,
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
pub struct SignatureUnlockDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub signature: SignatureDto,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignatureDto {
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
    pub index: u32,
    pub timestamp: u64,
    #[serde(rename = "parent1MessageId")]
    pub parent_1_message_id: String,
    #[serde(rename = "parent2MessageId")]
    pub parent_2_message_id: String,
    #[serde(rename = "inclusionMerkleProof")]
    pub inclusion_merkle_proof: String,
    #[serde(rename = "publicKeys")]
    pub public_keys: Vec<String>,
    pub signatures: Vec<String>,
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
    type Error = String;
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
    type Error = String;
    fn try_from(value: &MessageDto) -> Result<Self, Self::Error> {
        let mut builder = MessageBuilder::new()
            .with_network_id(
                value
                    .network_id
                    .parse::<u64>()
                    .map_err(|_| "invalid network id: expected an u64-string")?,
            )
            .with_parent1(value.parent_1_message_id.parse::<MessageId>().map_err(|_| {
                format!(
                    "invalid parent 1: expected a hex-string of length {}",
                    MESSAGE_ID_LENGTH * 2
                )
            })?)
            .with_parent2(value.parent_2_message_id.parse::<MessageId>().map_err(|_| {
                format!(
                    "invalid parent 2: expected a hex-string of length {}",
                    MESSAGE_ID_LENGTH * 2
                )
            })?)
            .with_nonce_provider(
                ConstantBuilder::new()
                    .with_value(
                        value
                            .nonce
                            .parse::<u64>()
                            .map_err(|_| "invalid nonce: expected an u64-string".to_string())?,
                    )
                    .finish(),
                0f64,
            );
        if let Some(p) = value.payload.as_ref() {
            builder = builder.with_payload(p.try_into()?);
        }
        Ok(builder.finish().map_err(|e| format!("invalid message: {}", e))?)
    }
}

// &Payload -> PayloadDto
impl TryFrom<&Payload> for PayloadDto {
    type Error = String;
    fn try_from(value: &Payload) -> Result<Self, Self::Error> {
        match value {
            Payload::Transaction(t) => Ok(PayloadDto::Transaction(t.try_into()?)),
            Payload::Milestone(m) => Ok(PayloadDto::Milestone(m.into())),
            Payload::Indexation(i) => Ok(PayloadDto::Indexation(i.into())),
            _ => Err("payload type not supported".to_string()),
        }
    }
}

// &PayloadDto -> Payload
impl TryFrom<&PayloadDto> for Payload {
    type Error = String;
    fn try_from(value: &PayloadDto) -> Result<Self, Self::Error> {
        match value {
            PayloadDto::Transaction(t) => Ok(Payload::Transaction(t.try_into()?)),
            PayloadDto::Milestone(m) => Ok(Payload::Milestone(m.try_into()?)),
            PayloadDto::Indexation(i) => Ok(Payload::Indexation(i.try_into()?)),
        }
    }
}

// &Box<Transaction> -> TransactionDto
impl TryFrom<&Box<TransactionPayload>> for TransactionDto {
    type Error = String;
    fn try_from(value: &Box<TransactionPayload>) -> Result<Self, Self::Error> {
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
impl TryFrom<&TransactionDto> for Box<TransactionPayload> {
    type Error = String;
    fn try_from(value: &TransactionDto) -> Result<Self, Self::Error> {
        let mut builder = TransactionPayload::builder().with_essence((&value.essence).try_into()?);
        for b in &value.unlock_blocks {
            builder = builder.add_unlock_block(b.try_into()?);
        }
        Ok(Box::new(
            builder
                .finish()
                .map_err(|e| format!("invalid transaction payload: {}", e))?,
        ))
    }
}

// &TransactionEssence -> TransactionEssenceDto
impl TryFrom<&TransactionPayloadEssence> for TransactionEssenceDto {
    type Error = String;
    fn try_from(value: &TransactionPayloadEssence) -> Result<Self, Self::Error> {
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
                Some(_) => {
                    return Err("invalid transaction essence: expected an optional indexation-payload".to_string())
                }
                None => None,
            },
        })
    }
}

// &TransactionEssenceDto -> TransactionEssence
impl TryFrom<&TransactionEssenceDto> for TransactionPayloadEssence {
    type Error = String;
    fn try_from(value: &TransactionEssenceDto) -> Result<Self, Self::Error> {
        let mut builder = TransactionPayloadEssence::builder();

        for i in &value.inputs {
            builder = builder.add_input(i.try_into()?);
        }

        for o in &value.outputs {
            builder = builder.add_output(o.try_into()?);
        }

        if let Some(p) = &value.payload {
            builder = builder.with_payload(Payload::Indexation((p).try_into()?));
        }

        Ok(builder
            .finish()
            .map_err(|e| format!("invalid transaction essence: {}", e))?)
    }
}

// &Input -> InputDto
impl TryFrom<&Input> for InputDto {
    type Error = String;
    fn try_from(value: &Input) -> Result<Self, Self::Error> {
        match value {
            Input::UTXO(u) => Ok(InputDto::UTXO(UtxoInputDto {
                kind: 0,
                transaction_id: u.output_id().transaction_id().to_string(),
                transaction_output_index: u.output_id().index(),
            })),
            _ => Err("input type not supported".to_string()),
        }
    }
}

// &InputDto -> Input
impl TryFrom<&InputDto> for Input {
    type Error = String;
    fn try_from(value: &InputDto) -> Result<Self, Self::Error> {
        match value {
            InputDto::UTXO(i) => Ok(Input::UTXO(
                UTXOInput::new(
                    i.transaction_id.parse::<TransactionId>().map_err(|_| {
                        format!(
                            "invalid transaction id: expected a hex-string of length {}",
                            TRANSACTION_ID_LENGTH * 2
                        )
                    })?,
                    i.transaction_output_index,
                )
                .map_err(|e| format!("invalid input: {}", e))?,
            )),
        }
    }
}

// &Output -> OutputDto
impl TryFrom<&Output> for OutputDto {
    type Error = String;
    fn try_from(value: &Output) -> Result<Self, Self::Error> {
        match value {
            Output::SignatureLockedSingle(s) => Ok(OutputDto::SignatureLockedSingle(SignatureLockedSingleOutputDto {
                kind: 0,
                address: s.address().try_into()?,
                amount: s.amount(),
            })),
            _ => Err("output type not supported".to_string()),
        }
    }
}

// &OutputDto -> Output
impl TryFrom<&OutputDto> for Output {
    type Error = String;
    fn try_from(value: &OutputDto) -> Result<Self, Self::Error> {
        match value {
            OutputDto::SignatureLockedSingle(s) => Ok(Output::SignatureLockedSingle(
                SignatureLockedSingleOutput::new((&s.address).try_into()?, s.amount)
                    // TODO unwrap
                    .unwrap(),
            )),
        }
    }
}

// &Address -> AddressDto
impl TryFrom<&Address> for AddressDto {
    type Error = String;
    fn try_from(value: &Address) -> Result<Self, Self::Error> {
        match value {
            Address::Ed25519(ed) => Ok(AddressDto::Ed25519(ed.into())),
            _ => Err("address type not supported".to_string()),
        }
    }
}

// &AddressDto -> Address
impl TryFrom<&AddressDto> for Address {
    type Error = String;
    fn try_from(value: &AddressDto) -> Result<Self, Self::Error> {
        match value {
            AddressDto::Ed25519(a) => Ok(Address::Ed25519(a.try_into()?)),
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
    type Error = String;
    fn try_from(value: &Ed25519AddressDto) -> Result<Self, Self::Error> {
        Ok(value.address.parse::<Ed25519Address>().map_err(|_| {
            format!(
                "invalid Ed25519 address: expected a hex-string of length {}",
                ED25519_ADDRESS_LENGTH * 2
            )
        })?)
    }
}

// &UnlockBlock -> UnlockBlockDto
impl TryFrom<&UnlockBlock> for UnlockBlockDto {
    type Error = String;
    fn try_from(value: &UnlockBlock) -> Result<Self, Self::Error> {
        match value {
            UnlockBlock::Signature(s) => match s {
                SignatureUnlock::Ed25519(ed) => Ok(UnlockBlockDto::Signature(SignatureUnlockDto {
                    kind: 0,
                    signature: SignatureDto::Ed25519(Ed25519SignatureDto {
                        kind: 1,
                        public_key: hex::encode(ed.public_key()),
                        signature: hex::encode(ed.signature()),
                    }),
                })),
                _ => Err("signature unlock type not supported".to_string()),
            },
            UnlockBlock::Reference(r) => Ok(UnlockBlockDto::Reference(ReferenceUnlockDto {
                kind: 1,
                index: r.index(),
            })),
            _ => Err("unlock block type not supported".to_string()),
        }
    }
}

// &UnlockBlockDto -> UnlockBlock
impl TryFrom<&UnlockBlockDto> for UnlockBlock {
    type Error = String;
    fn try_from(value: &UnlockBlockDto) -> Result<Self, Self::Error> {
        match value {
            UnlockBlockDto::Signature(s) => match &s.signature {
                SignatureDto::Ed25519(ed) => {
                    let mut public_key = [0u8; 32];
                    hex::decode_to_slice(&ed.public_key, &mut public_key).map_err(|_| {
                        "invalid public key in signature unlock block: expected a hex-string of length 64"
                    })?; // TODO access ED25519_PUBLIC_KEY_LENGTH when available
                    let signature = hex::decode(&ed.signature)
                        .map_err(|_| {
                            "invalid signature in signature unlock block: expected a hex-string of length 128"
                        })? // TODO access ED25519_SIGNATURE_LENGTH when available
                        .into_boxed_slice();
                    Ok(UnlockBlock::Signature(SignatureUnlock::Ed25519(Ed25519Signature::new(
                        public_key, signature,
                    ))))
                }
            },
            UnlockBlockDto::Reference(r) => Ok(UnlockBlock::Reference(
                ReferenceUnlock::new(r.index).map_err(|e| format!("invalid reference unlock block: {}", e))?,
            )),
        }
    }
}

// Box<Milestone> -> MilestoneDto
impl From<&Box<MilestonePayload>> for MilestoneDto {
    fn from(value: &Box<MilestonePayload>) -> Self {
        MilestoneDto {
            kind: 1,
            index: value.essence().index(),
            timestamp: value.essence().timestamp(),
            parent_1_message_id: value.essence().parent1().to_string(),
            parent_2_message_id: value.essence().parent2().to_string(),
            inclusion_merkle_proof: hex::encode(value.essence().merkle_proof()),
            public_keys: value.essence().public_keys().iter().map(hex::encode).collect(),
            signatures: value.signatures().iter().map(hex::encode).collect(),
        }
    }
}

// MilestoneDto -> Box<Milestone>
impl TryFrom<&MilestoneDto> for Box<MilestonePayload> {
    type Error = String;
    fn try_from(value: &MilestoneDto) -> Result<Self, Self::Error> {
        let essence = {
            let index = value.index;
            let timestamp = value.timestamp;
            let parent_1_message_id = value.parent_1_message_id.parse::<MessageId>().map_err(|_| {
                format!(
                    "invalid parent 1 in milestone essence: expected a hex-string of length {}",
                    MESSAGE_ID_LENGTH * 2
                )
            })?;
            let parent_2_message_id = value.parent_2_message_id.parse::<MessageId>().map_err(|_| {
                format!(
                    "invalid parent 2 in milestone essence: expected a hex-string of length {}",
                    MESSAGE_ID_LENGTH * 2
                )
            })?;
            let merkle_proof = {
                let mut buf = [0u8; MILESTONE_MERKLE_PROOF_LENGTH];
                hex::decode_to_slice(&value.inclusion_merkle_proof, &mut buf).map_err(|_| {
                    format!(
                        "invalid merkle proof in milestone essence: expected a hex-string of length {}",
                        MILESTONE_MERKLE_PROOF_LENGTH * 2
                    )
                })?;
                buf
            };
            let mut public_keys = Vec::new();
            for v in &value.public_keys {
                let key = {
                    let mut buf = [0u8; MILESTONE_PUBLIC_KEY_LENGTH];
                    hex::decode_to_slice(v, &mut buf).map_err(|_| {
                        format!(
                            "invalid public key in milestone essence: expected a hex-string of length {}",
                            MILESTONE_PUBLIC_KEY_LENGTH * 2
                        )
                    })?;
                    buf
                };
                public_keys.push(key);
            }
            MilestonePayloadEssence::new(
                index,
                timestamp,
                parent_1_message_id,
                parent_2_message_id,
                merkle_proof,
                public_keys,
            )
        };
        let mut signatures = Vec::new();
        for v in &value.signatures {
            signatures.push(
                hex::decode(v)
                    .map_err(|_| {
                        format!(
                            "invalid signature: expected a hex-string of length {}",
                            MILESTONE_SIGNATURE_LENGTH * 2
                        )
                    })?
                    .into_boxed_slice(),
            )
        }
        Ok(Box::new(MilestonePayload::new(essence, signatures)))
    }
}

impl From<&Box<IndexationPayload>> for IndexationDto {
    fn from(value: &Box<IndexationPayload>) -> Self {
        IndexationDto {
            kind: 2,
            index: value.index().to_owned(),
            data: hex::encode(value.data()),
        }
    }
}

// IndexationDto -> Box<Indexation>
impl TryFrom<&IndexationDto> for Box<IndexationPayload> {
    type Error = String;
    fn try_from(value: &IndexationDto) -> Result<Self, Self::Error> {
        Ok(Box::new(
            IndexationPayload::new(
                value.index.clone(),
                &hex::decode(value.data.clone())
                    .map_err(|_| "invalid data in indexation payload: expected a hex-string")?,
            )
            .map_err(|e| format!("invalid indexation payload: {}", e))?,
        ))
    }
}
