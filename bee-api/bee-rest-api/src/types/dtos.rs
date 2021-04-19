// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::Receipt;
use bee_message::{
    address::{Address, Ed25519Address, ED25519_ADDRESS_LENGTH},
    input::{Input, TreasuryInput, UtxoInput},
    milestone::MilestoneIndex,
    output::{Output, SignatureLockedDustAllowanceOutput, SignatureLockedSingleOutput, TreasuryOutput},
    parents::Parents,
    payload::{
        indexation::IndexationPayload,
        milestone::{
            MilestoneId, MilestonePayload, MilestonePayloadEssence, MILESTONE_MERKLE_PROOF_LENGTH,
            MILESTONE_PUBLIC_KEY_LENGTH, MILESTONE_SIGNATURE_LENGTH,
        },
        receipt::{MigratedFundsEntry, ReceiptPayload, TailTransactionHash},
        transaction::{Essence, RegularEssence, TransactionId, TransactionPayload, TRANSACTION_ID_LENGTH},
        treasury::TreasuryTransactionPayload,
        Payload,
    },
    signature::{Ed25519Signature, SignatureUnlock},
    unlock::{ReferenceUnlock, UnlockBlock, UnlockBlocks},
    Message, MessageBuilder, MessageId, MESSAGE_ID_LENGTH,
};
use bee_protocol::types::peer::Peer;

use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

use std::convert::{TryFrom, TryInto};

/// The message object that nodes gossip around in the network.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageDto {
    #[serde(rename = "networkId")]
    pub network_id: String,
    #[serde(rename = "parentMessageIds")]
    pub parents: Vec<String>,
    pub payload: Option<PayloadDto>,
    pub nonce: String,
}

/// Describes all the different payload types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PayloadDto {
    Transaction(Box<TransactionPayloadDto>),
    Milestone(Box<MilestonePayloadDto>),
    Indexation(Box<IndexationPayloadDto>),
    Receipt(Box<ReceiptPayloadDto>),
    TreasuryTransaction(Box<TreasuryTransactionPayloadDto>),
}

/// The payload type to define a value transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub essence: EssenceDto,
    #[serde(rename = "unlockBlocks")]
    pub unlock_blocks: Vec<UnlockBlockDto>,
}

/// Describes all the different essence types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EssenceDto {
    Regular(RegularEssenceDto),
}

/// Describes the essence data making up a transaction by defining its inputs and outputs and an optional payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegularEssenceDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub inputs: Vec<InputDto>,
    pub outputs: Vec<OutputDto>,
    pub payload: Option<PayloadDto>,
}

/// Describes all the different input types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputDto {
    Utxo(UtxoInputDto),
    Treasury(TreasuryInputDto),
}

/// Describes an input which references an unspent transaction output to consume.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtxoInputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "transactionOutputIndex")]
    pub transaction_output_index: u16,
}

/// Describes an input which references an unspent treasury output to consume.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreasuryInputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "milestoneId")]
    pub milestone_id: String,
}

/// Describes all the different output types.
#[derive(Clone, Debug)]
pub enum OutputDto {
    SignatureLockedSingle(SignatureLockedSingleOutputDto),
    SignatureLockedDustAllowance(SignatureLockedDustAllowanceOutputDto),
    Treasury(TreasuryOutputDto),
}

impl<'de> serde::Deserialize<'de> for OutputDto {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        Ok(match value.get("type").and_then(Value::as_u64).unwrap() as u8 {
            // TODO: cover all cases + handle unwraps
            SignatureLockedSingleOutput::KIND => {
                OutputDto::SignatureLockedSingle(SignatureLockedSingleOutputDto::deserialize(value).unwrap())
            }
            SignatureLockedDustAllowanceOutput::KIND => OutputDto::SignatureLockedDustAllowance(
                SignatureLockedDustAllowanceOutputDto::deserialize(value).unwrap(),
            ),
            TreasuryOutput::KIND => OutputDto::Treasury(TreasuryOutputDto::deserialize(value).unwrap()),
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
            OutputDto::SignatureLockedSingle(s) => TypedOutput {
                output: OutputDto_::T1(s),
            },
            OutputDto::SignatureLockedDustAllowance(s) => TypedOutput {
                output: OutputDto_::T2(s),
            },
            OutputDto::Treasury(t) => TypedOutput {
                output: OutputDto_::T3(t),
            },
        };
        output.serialize(serializer)
    }
}

/// Describes a deposit to a single address which is unlocked via a signature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureLockedSingleOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub address: AddressDto,
    pub amount: u64,
}

/// Output type for deposits that enables an address to receive dust outputs. It can be consumed as an input like a regular SigLockedSingleOutput.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureLockedDustAllowanceOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub address: AddressDto,
    pub amount: u64,
}

/// Describes all the different address types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddressDto {
    Ed25519(Ed25519AddressDto),
}

/// Describes an Ed25519 address.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ed25519AddressDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub address: String,
}

/// Describes a treasury output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreasuryOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub amount: u64,
}

/// Describes all the different unlock types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnlockBlockDto {
    Signature(SignatureUnlockDto),
    Reference(ReferenceUnlockDto),
}

/// Defines an unlock block containing signature(s) unlocking input(s).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureUnlockDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub signature: SignatureDto,
}

/// Describes all the different signature types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignatureDto {
    Ed25519(Ed25519SignatureDto),
}

/// Defines an Ed25519 signature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ed25519SignatureDto {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
}

/// References a previous unlock block in order to substitute the duplication of the same unlock block data for inputs which unlock through the same data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferenceUnlockDto {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "reference")]
    pub index: u16,
}

/// The payload type to define a milestone.
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
    #[serde(rename = "nextPoWScore")]
    pub next_pow_score: u32,
    #[serde(rename = "nextPoWScoreMilestoneIndex")]
    pub next_pow_score_milestone_index: u32,
    #[serde(rename = "publicKeys")]
    pub public_keys: Vec<String>,
    pub receipt: Option<PayloadDto>,
    pub signatures: Vec<String>,
}

/// The payload type to define a indexation payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexationPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: String,
    pub data: String,
}

/// The payload type to define a receipt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiptPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    #[serde(rename = "migratedAt")]
    pub migrated_at: u32,
    pub funds: Vec<MigratedFundsEntryDto>,
    pub transaction: PayloadDto,
    #[serde(rename = "final")]
    pub last: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigratedFundsEntryDto {
    #[serde(rename = "tailTransactionHash")]
    pub tail_transaction_hash: String,
    pub address: AddressDto,
    pub deposit: u64,
}

/// The payload type to define a treasury transaction.
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
                Parents::new(
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
                .map_err(|e| e.to_string())?,
            )
            .with_nonce_provider(
                value
                    .nonce
                    .parse::<u64>()
                    .map_err(|_| "invalid nonce: expected an u64-string".to_string())?,
                0f64,
            );
        if let Some(p) = value.payload.as_ref() {
            builder = builder.with_payload(p.try_into()?);
        }

        builder.finish().map_err(|e| format!("invalid message: {}", e))
    }
}

// &Payload -> PayloadDto
impl TryFrom<&Payload> for PayloadDto {
    type Error = String;

    fn try_from(value: &Payload) -> Result<Self, Self::Error> {
        match value {
            Payload::Transaction(t) => Ok(PayloadDto::Transaction(Box::new(TransactionPayloadDto::try_from(
                t.as_ref(),
            )?))),
            Payload::Milestone(m) => Ok(PayloadDto::Milestone(Box::new(MilestonePayloadDto::try_from(
                m.as_ref(),
            )?))),
            Payload::Indexation(i) => Ok(PayloadDto::Indexation(Box::new(IndexationPayloadDto::from(i.as_ref())))),
            _ => Err("payload type not supported".to_string()),
        }
    }
}

// &PayloadDto -> Payload
impl TryFrom<&PayloadDto> for Payload {
    type Error = String;

    fn try_from(value: &PayloadDto) -> Result<Self, Self::Error> {
        Ok(match value {
            PayloadDto::Transaction(t) => Payload::Transaction(Box::new(TransactionPayload::try_from(t.as_ref())?)),
            PayloadDto::Milestone(m) => Payload::Milestone(Box::new(MilestonePayload::try_from(m.as_ref())?)),
            PayloadDto::Indexation(i) => Payload::Indexation(Box::new(IndexationPayload::try_from(i.as_ref())?)),
            PayloadDto::Receipt(r) => Payload::Receipt(Box::new(ReceiptPayload::try_from(r.as_ref())?)),
            PayloadDto::TreasuryTransaction(t) => {
                Payload::TreasuryTransaction(Box::new(TreasuryTransactionPayload::try_from(t.as_ref())?))
            }
        })
    }
}

// &TransactionPayload -> TransactionPayloadDto
impl TryFrom<&TransactionPayload> for TransactionPayloadDto {
    type Error = String;

    fn try_from(value: &TransactionPayload) -> Result<Self, Self::Error> {
        Ok(TransactionPayloadDto {
            kind: TransactionPayload::KIND,
            essence: value.essence().try_into()?,
            unlock_blocks: value
                .unlock_blocks()
                .iter()
                .map(|u| u.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

// &TransactionPayloadDto -> TransactionPayload
impl TryFrom<&TransactionPayloadDto> for TransactionPayload {
    type Error = String;

    fn try_from(value: &TransactionPayloadDto) -> Result<Self, Self::Error> {
        let mut unlock_blocks = Vec::new();
        for b in &value.unlock_blocks {
            unlock_blocks.push(b.try_into()?);
        }
        let builder = TransactionPayload::builder()
            .with_essence((&value.essence).try_into()?)
            .with_unlock_blocks(UnlockBlocks::new(unlock_blocks).map_err(|e| e.to_string())?);

        builder
            .finish()
            .map_err(|e| format!("invalid transaction payload: {}", e))
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
            kind: RegularEssence::KIND,
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
                Some(Payload::Indexation(i)) => Some(PayloadDto::Indexation(Box::new(i.as_ref().into()))),
                Some(_) => {
                    return Err("invalid transaction essence: expected an optional indexation-payload".to_string());
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
            if let PayloadDto::Indexation(i) = p {
                builder = builder.with_payload(Payload::Indexation(Box::new((i.as_ref()).try_into()?)));
            } else {
                return Err("invalid transaction essence: expected an optional indexation-payload".to_string());
            }
        }

        builder
            .finish()
            .map_err(|e| format!("invalid transaction essence: {}", e))
    }
}

// &Input -> InputDto
impl TryFrom<&Input> for InputDto {
    type Error = String;

    fn try_from(value: &Input) -> Result<Self, Self::Error> {
        match value {
            Input::Utxo(u) => Ok(InputDto::Utxo(UtxoInputDto {
                kind: UtxoInput::KIND,
                transaction_id: u.output_id().transaction_id().to_string(),
                transaction_output_index: u.output_id().index(),
            })),
            Input::Treasury(t) => Ok(InputDto::Treasury(TreasuryInputDto {
                kind: TreasuryInput::KIND,
                milestone_id: t.milestone_id().to_string(),
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
            InputDto::Utxo(i) => Ok(Input::Utxo(
                UtxoInput::new(
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
                t.milestone_id
                    .parse::<MilestoneId>()
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
                kind: SignatureLockedSingleOutput::KIND,
                address: s.address().try_into()?,
                amount: s.amount(),
            })),
            Output::SignatureLockedDustAllowance(s) => Ok(OutputDto::SignatureLockedDustAllowance(
                SignatureLockedDustAllowanceOutputDto {
                    kind: SignatureLockedDustAllowanceOutput::KIND,
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
            kind: Ed25519Address::KIND,
            address: value.to_string(),
        }
    }
}

// &Ed25519AddressDto -> Ed25519Address
impl TryFrom<&Ed25519AddressDto> for Ed25519Address {
    type Error = String;

    fn try_from(value: &Ed25519AddressDto) -> Result<Self, Self::Error> {
        value.address.parse::<Ed25519Address>().map_err(|_| {
            format!(
                "invalid Ed25519 address: expected a hex-string of length {}",
                ED25519_ADDRESS_LENGTH * 2
            )
        })
    }
}

// &UnlockBlock -> UnlockBlockDto
impl TryFrom<&UnlockBlock> for UnlockBlockDto {
    type Error = String;

    fn try_from(value: &UnlockBlock) -> Result<Self, Self::Error> {
        match value {
            UnlockBlock::Signature(s) => match s {
                SignatureUnlock::Ed25519(ed) => Ok(UnlockBlockDto::Signature(SignatureUnlockDto {
                    kind: SignatureUnlock::KIND,
                    signature: SignatureDto::Ed25519(Ed25519SignatureDto {
                        kind: Ed25519Signature::KIND,
                        public_key: hex::encode(ed.public_key()),
                        signature: hex::encode(ed.signature()),
                    }),
                })),
                _ => Err("signature unlock type not supported".to_string()),
            },
            UnlockBlock::Reference(r) => Ok(UnlockBlockDto::Reference(ReferenceUnlockDto {
                kind: ReferenceUnlock::KIND,
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
                    hex::decode_to_slice(&ed.public_key, &mut public_key).map_err(
                        |_| "invalid public key in signature unlock block: expected a hex-string of length 64",
                    )?; // TODO access ED25519_PUBLIC_KEY_LENGTH when available
                    let signature = hex::decode(&ed.signature).map_err(
                        |_| "invalid signature in signature unlock block: expected a hex-string of length 128",
                    )?; // TODO access ED25519_SIGNATURE_LENGTH when available
                    Ok(UnlockBlock::Signature(SignatureUnlock::Ed25519(Ed25519Signature::new(
                        public_key,
                        signature.try_into().map_err(|_| "Invalid signature")?,
                    ))))
                }
            },
            UnlockBlockDto::Reference(r) => Ok(UnlockBlock::Reference(
                ReferenceUnlock::new(r.index).map_err(|e| format!("invalid reference unlock block: {}", e))?,
            )),
        }
    }
}

// MilestonePayload -> MilestonePayloadDto
impl TryFrom<&MilestonePayload> for MilestonePayloadDto {
    type Error = String;

    fn try_from(value: &MilestonePayload) -> Result<Self, Self::Error> {
        Ok(MilestonePayloadDto {
            kind: MilestonePayload::KIND,
            index: *value.essence().index(),
            timestamp: value.essence().timestamp(),
            parents: value.essence().parents().iter().map(|p| p.to_string()).collect(),
            inclusion_merkle_proof: hex::encode(value.essence().merkle_proof()),
            next_pow_score: value.essence().next_pow_score(),
            next_pow_score_milestone_index: value.essence().next_pow_score_milestone_index(),
            public_keys: value.essence().public_keys().iter().map(hex::encode).collect(),
            receipt: value.essence().receipt().map(TryInto::try_into).transpose()?,
            signatures: value.signatures().iter().map(hex::encode).collect(),
        })
    }
}

// &MilestonePayloadDto -> MilestonePayload
impl TryFrom<&MilestonePayloadDto> for MilestonePayload {
    type Error = String;

    fn try_from(value: &MilestonePayloadDto) -> Result<Self, Self::Error> {
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
            let next_pow_score = value.next_pow_score;
            let next_pow_score_milestone_index = value.next_pow_score_milestone_index;
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
            let receipt = if let Some(receipt) = value.receipt.as_ref() {
                Some(receipt.try_into()?)
            } else {
                None
            };
            MilestonePayloadEssence::new(
                MilestoneIndex(index),
                timestamp,
                Parents::new(parent_ids).map_err(|e| e.to_string())?,
                merkle_proof,
                next_pow_score,
                next_pow_score_milestone_index,
                public_keys,
                receipt,
            )
            .map_err(|e| e.to_string())?
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
                    .try_into()
                    .map_err(|_| "Invalid signatures".to_string())?,
            )
        }

        MilestonePayload::new(essence, signatures).map_err(|e| e.to_string())
    }
}

// &IndexationPayload -> IndexationPayloadDto
impl From<&IndexationPayload> for IndexationPayloadDto {
    fn from(value: &IndexationPayload) -> Self {
        IndexationPayloadDto {
            kind: IndexationPayload::KIND,
            index: hex::encode(value.index()),
            data: hex::encode(value.data()),
        }
    }
}

// &IndexationPayloadDto -> IndexationPayload
impl TryFrom<&IndexationPayloadDto> for IndexationPayload {
    type Error = String;

    fn try_from(value: &IndexationPayloadDto) -> Result<Self, Self::Error> {
        IndexationPayload::new(
            &hex::decode(value.index.clone())
                .map_err(|_| "invalid index in indexation payload: expected a hex-string")?,
            &hex::decode(value.data.clone())
                .map_err(|_| "invalid data in indexation payload: expected a hex-string")?,
        )
        .map_err(|e| format!("invalid indexation payload: {}", e))
    }
}

// &ReceiptPayload -> ReceiptPayloadDto
impl TryFrom<&ReceiptPayload> for ReceiptPayloadDto {
    type Error = String;

    fn try_from(value: &ReceiptPayload) -> Result<Self, Self::Error> {
        Ok(ReceiptPayloadDto {
            kind: ReceiptPayload::KIND,
            migrated_at: *value.migrated_at(),
            last: value.last(),
            funds: value
                .funds()
                .iter()
                .map(|m| m.try_into())
                .collect::<Result<Vec<MigratedFundsEntryDto>, _>>()?,
            transaction: value.transaction().try_into()?,
        })
    }
}

// &ReceiptPayloadDto -> ReceiptPayload
impl TryFrom<&ReceiptPayloadDto> for ReceiptPayload {
    type Error = String;

    fn try_from(value: &ReceiptPayloadDto) -> Result<Self, Self::Error> {
        let receipt = ReceiptPayload::new(
            MilestoneIndex(value.migrated_at),
            value.last,
            value
                .funds
                .iter()
                .map(|m| m.try_into())
                .collect::<Result<Vec<MigratedFundsEntry>, _>>()?,
            (&value.transaction).try_into()?,
        )
        .map_err(|e| format!("invalid receipt payload: {}", e))?;

        Ok(receipt)
    }
}

// &MigratedFundsEntry -> MigratedFundsEntryDto
impl TryFrom<&MigratedFundsEntry> for MigratedFundsEntryDto {
    type Error = String;

    fn try_from(value: &MigratedFundsEntry) -> Result<Self, Self::Error> {
        Ok(MigratedFundsEntryDto {
            tail_transaction_hash: hex::encode(value.tail_transaction_hash().as_ref()),
            address: value.output().address().try_into()?,
            deposit: value.output().amount(),
        })
    }
}

// &MigratedFundsEntryDto -> MigratedFundsEntry
impl TryFrom<&MigratedFundsEntryDto> for MigratedFundsEntry {
    type Error = String;

    fn try_from(value: &MigratedFundsEntryDto) -> Result<Self, Self::Error> {
        let entry = MigratedFundsEntry::new(
            TailTransactionHash::new(
                hex::decode(value.tail_transaction_hash.clone())
                    .map_err(|e| format!("invalid tail transaction hash: {:?}", e))?
                    .try_into()
                    .map_err(|e| format!("invalid tail transaction hash: {:?}", e))?,
            )
            .map_err(|e| format!("invalid tail transaction hash: {}", e))?,
            SignatureLockedSingleOutput::new((&value.address).try_into()?, value.deposit)
                .map_err(|e| format!("invalid address or amount: {}", e))?,
        )
        .map_err(|e| format!("invalid migrated funds entry: {}", e))?;
        Ok(entry)
    }
}

// &TreasuryTransactionPayload -> TreasuryTransactionPayloadDto
impl TryFrom<&TreasuryTransactionPayload> for TreasuryTransactionPayloadDto {
    type Error = String;

    fn try_from(value: &TreasuryTransactionPayload) -> Result<Self, Self::Error> {
        Ok(TreasuryTransactionPayloadDto {
            kind: TreasuryTransactionPayload::KIND,
            input: value.input().try_into()?,
            output: value.output().try_into()?,
        })
    }
}

// &TreasuryTransactionDto -> TreasuryTransactionPayload
impl TryFrom<&TreasuryTransactionPayloadDto> for TreasuryTransactionPayload {
    type Error = String;

    fn try_from(value: &TreasuryTransactionPayloadDto) -> Result<Self, Self::Error> {
        let input: Input = (&value.input)
            .try_into()
            .map_err(|_| "invalid input in treasury transaction payload: expected a treasury input")?;
        let output: Output = (&value.output)
            .try_into()
            .map_err(|_| "invalid output in treasury transaction payload: expected a treasury output")?;

        TreasuryTransactionPayload::new(input, output)
            .map_err(|e| format!("invalid treasury transaction payload: {}", e))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerDto {
    pub id: String,
    #[serde(rename = "multiAddresses")]
    pub multi_addresses: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub relation: RelationDto,
    pub connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
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

impl From<&Peer> for PeerDto {
    fn from(peer: &Peer) -> Self {
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
            connected: peer.is_connected(),
            gossip: Some(GossipDto {
                heartbeat: HeartbeatDto {
                    solid_milestone_index: *peer.solid_milestone_index(),
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
                    dropped_packets: 0,
                },
            }),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiptDto {
    pub receipt: ReceiptPayloadDto,
    #[serde(rename = "milestoneIndex")]
    pub milestone_index: u32,
}

impl TryFrom<Receipt> for ReceiptDto {
    type Error = String;

    fn try_from(value: Receipt) -> Result<Self, Self::Error> {
        Ok(ReceiptDto {
            receipt: value.inner().try_into().map_err(|_| "Invalid receipt".to_owned())?,
            milestone_index: **value.included_in(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LedgerInclusionStateDto {
    #[serde(rename = "conflicting")]
    Conflicting,
    #[serde(rename = "included")]
    Included,
    #[serde(rename = "noTransaction")]
    NoTransaction,
}
