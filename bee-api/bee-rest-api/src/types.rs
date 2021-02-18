// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{payload::receipt::ReceiptPayload, prelude::*};
use bee_pow::providers::{ConstantBuilder, ProviderBuilder};
use bee_protocol::{Peer, PeerManager};
use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize, Serializer};

use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageDto {
    #[serde(rename = "networkId")]
    pub network_id: String,
    #[serde(rename = "parentMessageIds")]
    pub parents: Vec<String>,
    pub payload: Option<PayloadDto>,
    pub nonce: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PayloadDto {
    Transaction(Box<TransactionPayloadDto>),
    Milestone(Box<MilestonePayloadDto>),
    Indexation(Box<IndexationPayloadDto>),
    Receipt(Box<ReceiptPayloadDto>),
    TreasuryTransaction(Box<TreasuryTransactionPayloadDto>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub essence: EssenceDto,
    #[serde(rename = "unlockBlocks")]
    pub unlock_blocks: Vec<UnlockBlockDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EssenceDto {
    Regular(RegularEssenceDto),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegularEssenceDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub inputs: Vec<InputDto>,
    pub outputs: Vec<OutputDto>,
    pub payload: Option<PayloadDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputDto {
    UTXO(UTXOInputDto),
    Treasury(TreasuryInputDto),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UTXOInputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "transactionOutputIndex")]
    pub transaction_output_index: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreasuryInputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "transactionId")]
    pub message_id: String,
}

#[derive(Clone, Debug)]
pub enum OutputDto {
    SignatureLockedSingle(SignatureLockedSingleOutputDto),
    SignatureLockedDustAllowance(SignatureLockedDustAllowanceOutputDto),
    Treasury(TreasuryOutputDto),
}

impl<'de> serde::Deserialize<'de> for OutputDto {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        Ok(match value.get("type").and_then(Value::as_u8).unwrap() {
            // TODO: cover all cases + handle unwraps
            1 => OutputDto::SignatureLockedSingle(SignatureLockedSingleOutputDto::deserialize(value).unwrap()),
            2 => OutputDto::SignatureLockedDustAllowance(SignatureLockedDustAllowanceOutputDto::deserialize(value).unwrap()),
            type_ => panic!("unsupported type {:?}", type_),
        })
    }
}

impl Serialize for OutputDto {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        #[derive(Serialize)]
        #[serde(untagged)]
        enum OutputDto_<'a> {
            T1(&'a SignatureLockedSingleOutputDto),
            T2(&'a SignatureLockedDustAllowanceOutputDto),
            T3(&'a TreasuryOutputDto),
        }
        #[derive(Serialize)]
        struct TypedOutput<'a> {
            #[serde(flatten)]
            output: OutputDto_<'a>,
        }
        let output = match self {
            OutputDto::SignatureLockedSingle(s) => TypedOutput { output: OutputDto_::T1(s) },
            OutputDto::SignatureLockedDustAllowance(s) => TypedOutput { output: OutputDto_::T2(s) },
            OutputDto::Treasury(t) => TypedOutput { output: OutputDto_::T3(t) },
        };
        output.serialize(serializer)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureLockedSingleOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub address: AddressDto,
    pub amount: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureLockedDustAllowanceOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
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
    pub kind: u8,
    pub address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreasuryOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub amount: u64,
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
    pub kind: u8,
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
    pub kind: u8,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferenceUnlockDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub index: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilestonePayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: u32,
    pub timestamp: u64,
    #[serde(rename = "parentMessageIds")]
    pub parents: Vec<String>,
    #[serde(rename = "inclusionMerkleProof")]
    pub inclusion_merkle_proof: String,
    #[serde(rename = "publicKeys")]
    pub public_keys: Vec<String>,
    pub receipt: Option<PayloadDto>,
    pub signatures: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexationPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: String,
    pub data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiptPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: u32,
    pub last: bool,
    pub funds: Vec<MigratedFundsEntryDto>,
    pub transaction: PayloadDto,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigratedFundsEntryDto {
    tail_transaction_hash: Box<[u8]>,
    address: AddressDto,
    amount: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreasuryTransactionPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub input: InputDto,
    pub output: OutputDto,
}

// &Message -> MessageDto
impl TryFrom<&Message> for MessageDto {
    type Error = String;
    fn try_from(value: &Message) -> Result<Self, Self::Error> {
        Ok(MessageDto {
            network_id: value.network_id().to_string(),
            parents: value.parents().iter().map(|p| p.to_string()).collect(),
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
            .with_parents(
                value
                    .parents
                    .iter()
                    .map(|m| {
                        m.parse::<MessageId>().map_err(|_| {
                            format!(
                                "invalid parent: expected a hex-string of length {}",
                                MESSAGE_ID_LENGTH * 2
                            )
                        })
                    })
                    .collect::<Result<Vec<MessageId>, String>>()?,
            )
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
                None,
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
            Payload::Milestone(m) => Ok(PayloadDto::Milestone(m.try_into()?)),
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
            PayloadDto::Receipt(r) => Ok(Payload::Receipt(r.try_into()?)),
            PayloadDto::TreasuryTransaction(t) => Ok(Payload::TreasuryTransaction(t.try_into()?)),
        }
    }
}

// &Box<Transaction> -> Box<TransactionDto>
impl TryFrom<&Box<TransactionPayload>> for Box<TransactionPayloadDto> {
    type Error = String;
    fn try_from(value: &Box<TransactionPayload>) -> Result<Self, Self::Error> {
        Ok(Box::new(TransactionPayloadDto {
            kind: 0,
            essence: value.essence().try_into()?,
            unlock_blocks: value
                .unlock_blocks()
                .iter()
                .map(|u| u.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        }))
    }
}

// &TransactionDto -> Box<Transaction>
impl TryFrom<&Box<TransactionPayloadDto>> for Box<TransactionPayload> {
    type Error = String;
    fn try_from(value: &Box<TransactionPayloadDto>) -> Result<Self, Self::Error> {
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

// &Essence -> EssenceDto
impl TryFrom<&Essence> for EssenceDto {
    type Error = String;
    fn try_from(value: &Essence) -> Result<Self, Self::Error> {
        match value {
            Essence::Regular(r) => Ok(EssenceDto::Regular(r.try_into()?)),
            _ => Err("essence type not supported".to_string()),
        }
    }
}

// &EssenceDto -> Essence
impl TryFrom<&EssenceDto> for Essence {
    type Error = String;
    fn try_from(value: &EssenceDto) -> Result<Self, Self::Error> {
        match value {
            EssenceDto::Regular(r) => Ok(Essence::Regular(r.try_into()?)),
        }
    }
}

// &RegularEssence -> RegularEssenceDto
impl TryFrom<&RegularEssence> for RegularEssenceDto {
    type Error = String;
    fn try_from(value: &RegularEssence) -> Result<Self, Self::Error> {
        Ok(RegularEssenceDto {
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
                Some(Payload::Indexation(i)) => Some(PayloadDto::Indexation(i.into())),
                Some(_) => {
                    return Err("invalid transaction essence: expected an optional indexation-payload".to_string())
                }
                None => None,
            },
        })
    }
}

// &RegularEssenceDto -> RegularEssence
impl TryFrom<&RegularEssenceDto> for RegularEssence {
    type Error = String;
    fn try_from(value: &RegularEssenceDto) -> Result<Self, Self::Error> {
        let mut builder = RegularEssence::builder();

        for i in &value.inputs {
            builder = builder.add_input(i.try_into()?);
        }

        for o in &value.outputs {
            builder = builder.add_output(o.try_into()?);
        }

        if let Some(p) = &value.payload {
            if let &PayloadDto::Indexation(i) = &p {
                builder = builder.with_payload(Payload::Indexation((i).try_into()?));
            } else {
                return Err("invalid transaction essence: expected an optional indexation-payload".to_string());
            }
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
            Input::UTXO(u) => Ok(InputDto::UTXO(UTXOInputDto {
                kind: 0,
                transaction_id: u.output_id().transaction_id().to_string(),
                transaction_output_index: u.output_id().index(),
            })),
            Input::Treasury(t) => Ok(InputDto::Treasury(TreasuryInputDto {
                kind: 1,
                message_id: t.message_id().to_string(),
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
            InputDto::Treasury(t) => Ok(Input::Treasury(
                t.message_id
                    .parse::<MessageId>()
                    .map_err(|e| format!("invalid treasury input: {}", e))?
                    .into(),
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
            Output::SignatureLockedDustAllowance(s) => Ok(OutputDto::SignatureLockedDustAllowance(
                SignatureLockedDustAllowanceOutputDto {
                    kind: 1,
                    address: s.address().try_into()?,
                    amount: s.amount(),
                },
            )),
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
            OutputDto::SignatureLockedDustAllowance(s) => Ok(Output::SignatureLockedDustAllowance(
                SignatureLockedDustAllowanceOutput::new((&s.address).try_into()?, s.amount)
                    // TODO unwrap
                    .unwrap(),
            )),
            OutputDto::Treasury(t) => Ok(Output::Treasury(
                TreasuryOutput::new(t.amount)
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
impl TryFrom<&Box<MilestonePayload>> for Box<MilestonePayloadDto> {
    type Error = String;
    fn try_from(value: &Box<MilestonePayload>) -> Result<Self, Self::Error> {
        Ok(Box::new(MilestonePayloadDto {
            kind: 1,
            index: value.essence().index(),
            timestamp: value.essence().timestamp(),
            parents: value.essence().parents().iter().map(|p| p.to_string()).collect(),
            inclusion_merkle_proof: hex::encode(value.essence().merkle_proof()),
            public_keys: value.essence().public_keys().iter().map(hex::encode).collect(),
            receipt: value.essence().receipt().map(TryInto::try_into).transpose()?,
            signatures: value.signatures().iter().map(hex::encode).collect(),
        }))
    }
}

// &Box<MilestoneDto> -> Box<Milestone>
impl TryFrom<&Box<MilestonePayloadDto>> for Box<MilestonePayload> {
    type Error = String;
    fn try_from(value: &Box<MilestonePayloadDto>) -> Result<Self, Self::Error> {
        let essence = {
            let index = value.index;
            let timestamp = value.timestamp;
            let mut parent_ids = Vec::new();
            for msg_id in &value.parents {
                parent_ids.push(msg_id.parse::<MessageId>().map_err(|_| {
                    format!(
                        "invalid parent in milestone essence: expected a hex-string of length {}",
                        MESSAGE_ID_LENGTH * 2
                    )
                })?);
            }
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
            MilestonePayloadEssence::new(index, timestamp, parent_ids, merkle_proof, public_keys)
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

// &Box<IndexationPayload> -> Box<IndexationDto>
impl From<&Box<IndexationPayload>> for Box<IndexationPayloadDto> {
    fn from(value: &Box<IndexationPayload>) -> Self {
        Box::new(IndexationPayloadDto {
            kind: 2,
            index: value.index().to_owned(),
            data: hex::encode(value.data()),
        })
    }
}

// &Box<IndexationDto> -> Box<IndexationPayload>
impl TryFrom<&Box<IndexationPayloadDto>> for Box<IndexationPayload> {
    type Error = String;
    fn try_from(value: &Box<IndexationPayloadDto>) -> Result<Self, Self::Error> {
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

// &Box<ReceiptPayload> -> Box<ReceiptDto>
impl TryFrom<&Box<ReceiptPayload>> for Box<ReceiptPayloadDto> {
    type Error = String;
    fn try_from(value: &Box<ReceiptPayload>) -> Result<Self, Self::Error> {
        Ok(Box::new(ReceiptPayloadDto {
            kind: 3,
            index: value.index(),
            last: value.last(),
            funds: value
                .funds()
                .iter()
                .map(|m| m.try_into())
                .collect::<Result<Vec<MigratedFundsEntryDto>, _>>()?,
            transaction: value.transaction().try_into()?,
        }))
    }
}

// &Box<ReceiptDto> -> Box<ReceiptPayload>
impl TryFrom<&Box<ReceiptPayloadDto>> for Box<ReceiptPayload> {
    type Error = String;
    fn try_from(value: &Box<ReceiptPayloadDto>) -> Result<Self, Self::Error> {
        let receipt = ReceiptPayload::new(
            value.index,
            value.last,
            value
                .funds
                .iter()
                .map(|m| m.try_into())
                .collect::<Result<Vec<MigratedFundsEntry>, _>>()?,
            (&value.transaction).try_into()?,
        )
        .map_err(|e| format!("invalid receipt payload: {}", e))?;
        Ok(Box::new(receipt))
    }
}

// &MigratedFundsEntry -> MigratedFundsEntryDto
impl TryFrom<&MigratedFundsEntry> for MigratedFundsEntryDto {
    type Error = String;
    fn try_from(value: &MigratedFundsEntry) -> Result<Self, Self::Error> {
        Ok(MigratedFundsEntryDto {
            tail_transaction_hash: Box::new(value.tail_transaction_hash().clone()),
            address: value.address().try_into()?,
            amount: value.amount(),
        })
    }
}

// &MigratedFundsEntryDto -> MigratedFundsEntry
impl TryFrom<&MigratedFundsEntryDto> for MigratedFundsEntry {
    type Error = String;
    fn try_from(value: &MigratedFundsEntryDto) -> Result<Self, Self::Error> {
        let entry = MigratedFundsEntry::new(
            value
                .tail_transaction_hash
                .as_ref()
                .try_into()
                .map_err(|e| format!("invalid tail transaction hash: {}", e))?,
            (&value.address).try_into()?,
            value.amount,
        )
        .map_err(|e| format!("invalid migrated funds entry: {}", e))?;
        Ok(entry)
    }
}

// &Box<ReceiptPayload> -> Box<ReceiptDto>§
impl TryFrom<&Box<TreasuryTransactionPayload>> for Box<TreasuryTransactionPayloadDto> {
    type Error = String;
    fn try_from(value: &Box<TreasuryTransactionPayload>) -> Result<Self, Self::Error> {
        Ok(Box::new(TreasuryTransactionPayloadDto {
            kind: 4,
            input: value.input().try_into()?,
            output: value.output().try_into()?,
        }))
    }
}

// &Box<TreasuryTransactionDto> -> Box<TreasuryTransactionPayload>
impl TryFrom<&Box<TreasuryTransactionPayloadDto>> for Box<TreasuryTransactionPayload> {
    type Error = String;
    fn try_from(value: &Box<TreasuryTransactionPayloadDto>) -> Result<Self, Self::Error> {
        let input: Input = (&value.input)
            .try_into()
            .map_err(|_| "invalid input in treasury transaction payload: expected a treasury input")?;
        let output: Output = (&value.output)
            .try_into()
            .map_err(|_| "invalid output in treasury transaction payload: expected a treasury output")?;
        Ok(Box::new(
            TreasuryTransactionPayload::new(input, output)
                .map_err(|e| format!("invalid treasury transaction payload: {}", e))?,
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerDto {
    pub id: String,
    #[serde(rename = "multiAddresses")]
    pub multi_addresses: Vec<String>,
    pub alias: Option<String>,
    pub relation: RelationDto,
    pub connected: bool,
    pub gossip: Option<GossipDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GossipDto {
    pub heartbeat: HeartbeatDto,
    pub metrics: MetricsDto,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RelationDto {
    #[serde(rename = "known")]
    Known,
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "discovered")]
    Discovered,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HeartbeatDto {
    #[serde(rename = "solidMilestoneIndex")]
    pub solid_milestone_index: u32,
    #[serde(rename = "prunedMilestoneIndex")]
    pub pruned_milestone_index: u32,
    #[serde(rename = "latestMilestoneIndex")]
    pub latest_milestone_index: u32,
    #[serde(rename = "connectedNeighbors")]
    pub connected_neighbors: u8,
    #[serde(rename = "syncedNeighbors")]
    pub synced_neighbors: u8,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MetricsDto {
    #[serde(rename = "newMessages")]
    pub new_messages: u64,
    #[serde(rename = "receivedMessages")]
    pub received_messages: u64,
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

// &Arc<Peer> -> PeerDto // TODO: can not implement `From` conversion since it's dependent of the `PeerManager`.
pub async fn peer_to_peer_dto(peer: &Arc<Peer>, peer_manager: &ResourceHandle<PeerManager>) -> PeerDto {
    PeerDto {
        id: peer.id().to_string(),
        alias: Some(peer.alias().to_string()),
        multi_addresses: vec![peer.address().to_string()],
        relation: {
            if peer.relation().is_known() {
                RelationDto::Known
            } else if peer.relation().is_unknown() {
                RelationDto::Unknown
            } else {
                RelationDto::Discovered
            }
        },
        connected: peer_manager.is_connected(peer.id()).await,
        gossip: Some(GossipDto {
            heartbeat: HeartbeatDto {
                solid_milestone_index: *peer.latest_solid_milestone_index(),
                pruned_milestone_index: *peer.pruned_index(),
                latest_milestone_index: *peer.latest_milestone_index(),
                connected_neighbors: peer.connected_peers(),
                synced_neighbors: peer.synced_peers(),
            },
            metrics: MetricsDto {
                new_messages: peer.metrics().new_messages(),
                received_messages: peer.metrics().messages_received(),
                known_messages: peer.metrics().known_messages(),
                received_message_requests: peer.metrics().message_requests_received(),
                received_milestone_requests: peer.metrics().milestone_requests_received(),
                received_heartbeats: peer.metrics().heartbeats_received(),
                sent_messages: peer.metrics().messages_sent(),
                sent_message_requests: peer.metrics().message_requests_sent(),
                sent_milestone_requests: peer.metrics().milestone_requests_sent(),
                sent_heartbeats: peer.metrics().heartbeats_sent(),
                dropped_packets: peer.metrics().invalid_packets(), // TODO dropped_packets == invalid_packets?
            },
        }),
    }
}
