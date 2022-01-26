// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::error::Error;

use bee_ledger::types::Receipt;
use bee_message::{
    address::{Address, AliasAddress, Ed25519Address, NftAddress},
    input::{Input, TreasuryInput, UtxoInput},
    milestone::MilestoneIndex,
    output::{
        feature_block::{FeatureBlock, IssuerFeatureBlock, MetadataFeatureBlock, SenderFeatureBlock, TagFeatureBlock},
        unlock_condition::{
            AddressUnlockCondition, DustDepositReturnUnlockCondition, ExpirationUnlockCondition,
            GovernorAddressUnlockCondition, StateControllerAddressUnlockCondition, TimelockUnlockCondition,
            UnlockCondition,
        },
        AliasId, AliasOutput, AliasOutputBuilder, ExtendedOutput, ExtendedOutputBuilder, FoundryOutput,
        FoundryOutputBuilder, NativeToken, NftId, NftOutput, NftOutputBuilder, Output, TokenId, TokenScheme,
        TreasuryOutput,
    },
    parent::Parents,
    payload::{
        milestone::{MilestoneEssence, MilestoneId, MilestonePayload},
        receipt::{MigratedFundsEntry, ReceiptPayload, TailTransactionHash},
        tagged_data::TaggedDataPayload,
        transaction::{RegularTransactionEssence, TransactionEssence, TransactionId, TransactionPayload},
        treasury::TreasuryTransactionPayload,
        Payload,
    },
    signature::{Ed25519Signature, Signature},
    unlock_block::{
        AliasUnlockBlock, NftUnlockBlock, ReferenceUnlockBlock, SignatureUnlockBlock, UnlockBlock, UnlockBlocks,
    },
    Message, MessageBuilder, MessageId,
};
#[cfg(feature = "peer")]
use bee_protocol::types::peer::Peer;

use primitive_types::U256;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

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

impl From<&Message> for MessageDto {
    fn from(value: &Message) -> Self {
        MessageDto {
            network_id: value.network_id().to_string(),
            parents: value.parents().iter().map(|p| p.to_string()).collect(),
            payload: value.payload().map(Into::into),
            nonce: value.nonce().to_string(),
        }
    }
}

impl TryFrom<&MessageDto> for Message {
    type Error = Error;

    fn try_from(value: &MessageDto) -> Result<Self, Self::Error> {
        let mut builder = MessageBuilder::new()
            .with_network_id(
                value
                    .network_id
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidField("networkId"))?,
            )
            .with_parents(Parents::new(
                value
                    .parents
                    .iter()
                    .map(|m| {
                        m.parse::<MessageId>()
                            .map_err(|_| Error::InvalidField("parentMessageIds"))
                    })
                    .collect::<Result<Vec<MessageId>, Error>>()?,
            )?)
            .with_nonce_provider(
                value.nonce.parse::<u64>().map_err(|_| Error::InvalidField("nonce"))?,
                0f64,
            );
        if let Some(p) = value.payload.as_ref() {
            builder = builder.with_payload(p.try_into()?);
        }

        Ok(builder.finish()?)
    }
}

/// Describes all the different payload types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PayloadDto {
    Transaction(Box<TransactionPayloadDto>),
    Milestone(Box<MilestonePayloadDto>),
    Receipt(Box<ReceiptPayloadDto>),
    TreasuryTransaction(Box<TreasuryTransactionPayloadDto>),
    TaggedData(Box<TaggedDataPayloadDto>),
}

impl From<&Payload> for PayloadDto {
    fn from(value: &Payload) -> Self {
        match value {
            Payload::Transaction(p) => PayloadDto::Transaction(Box::new(TransactionPayloadDto::from(p.as_ref()))),
            Payload::Milestone(p) => PayloadDto::Milestone(Box::new(MilestonePayloadDto::from(p.as_ref()))),
            Payload::Receipt(p) => PayloadDto::Receipt(Box::new(ReceiptPayloadDto::from(p.as_ref()))),
            Payload::TreasuryTransaction(p) => {
                PayloadDto::TreasuryTransaction(Box::new(TreasuryTransactionPayloadDto::from(p.as_ref())))
            }
            Payload::TaggedData(p) => PayloadDto::TaggedData(Box::new(TaggedDataPayloadDto::from(p.as_ref()))),
        }
    }
}

impl TryFrom<&PayloadDto> for Payload {
    type Error = Error;
    fn try_from(value: &PayloadDto) -> Result<Self, Self::Error> {
        Ok(match value {
            PayloadDto::Transaction(p) => Payload::Transaction(Box::new(TransactionPayload::try_from(p.as_ref())?)),
            PayloadDto::Milestone(p) => Payload::Milestone(Box::new(MilestonePayload::try_from(p.as_ref())?)),
            PayloadDto::Receipt(p) => Payload::Receipt(Box::new(ReceiptPayload::try_from(p.as_ref())?)),
            PayloadDto::TreasuryTransaction(p) => {
                Payload::TreasuryTransaction(Box::new(TreasuryTransactionPayload::try_from(p.as_ref())?))
            }
            PayloadDto::TaggedData(p) => Payload::TaggedData(Box::new(TaggedDataPayload::try_from(p.as_ref())?)),
        })
    }
}

/// The payload type to define a value transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub essence: TransactionEssenceDto,
    #[serde(rename = "unlockBlocks")]
    pub unlock_blocks: Vec<UnlockBlockDto>,
}

impl From<&TransactionPayload> for TransactionPayloadDto {
    fn from(value: &TransactionPayload) -> Self {
        TransactionPayloadDto {
            kind: TransactionPayload::KIND,
            essence: value.essence().into(),
            unlock_blocks: value.unlock_blocks().iter().map(Into::into).collect::<Vec<_>>(),
        }
    }
}

impl TryFrom<&TransactionPayloadDto> for TransactionPayload {
    type Error = Error;

    fn try_from(value: &TransactionPayloadDto) -> Result<Self, Self::Error> {
        let mut unlock_blocks = Vec::new();
        for b in &value.unlock_blocks {
            unlock_blocks.push(b.try_into()?);
        }
        let builder = TransactionPayload::builder()
            .with_essence((&value.essence).try_into()?)
            .with_unlock_blocks(UnlockBlocks::new(unlock_blocks)?);

        Ok(builder.finish()?)
    }
}

/// Describes all the different essence types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransactionEssenceDto {
    Regular(RegularTransactionEssenceDto),
}

impl From<&TransactionEssence> for TransactionEssenceDto {
    fn from(value: &TransactionEssence) -> Self {
        match value {
            TransactionEssence::Regular(r) => TransactionEssenceDto::Regular(r.into()),
        }
    }
}

impl TryFrom<&TransactionEssenceDto> for TransactionEssence {
    type Error = Error;

    fn try_from(value: &TransactionEssenceDto) -> Result<Self, Self::Error> {
        match value {
            TransactionEssenceDto::Regular(r) => Ok(TransactionEssence::Regular(r.try_into()?)),
        }
    }
}

/// Describes the essence data making up a transaction by defining its inputs and outputs and an optional payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegularTransactionEssenceDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub inputs: Vec<InputDto>,
    pub outputs: Vec<OutputDto>,
    pub payload: Option<PayloadDto>,
}

impl From<&RegularTransactionEssence> for RegularTransactionEssenceDto {
    fn from(value: &RegularTransactionEssence) -> Self {
        RegularTransactionEssenceDto {
            kind: RegularTransactionEssence::KIND,
            inputs: value.inputs().iter().map(Into::into).collect::<Vec<_>>(),
            outputs: value.outputs().iter().map(Into::into).collect::<Vec<_>>(),
            payload: match value.payload() {
                Some(Payload::TaggedData(i)) => Some(PayloadDto::TaggedData(Box::new(i.as_ref().into()))),
                Some(_) => unimplemented!(),
                None => None,
            },
        }
    }
}

impl TryFrom<&RegularTransactionEssenceDto> for RegularTransactionEssence {
    type Error = Error;

    fn try_from(value: &RegularTransactionEssenceDto) -> Result<Self, Self::Error> {
        let inputs = value
            .inputs
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Input>, Self::Error>>()?;
        let outputs = value
            .outputs
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Output>, Self::Error>>()?;

        let mut builder = RegularTransactionEssence::builder()
            .with_inputs(inputs)
            .with_outputs(outputs);
        builder = if let Some(p) = &value.payload {
            if let PayloadDto::TaggedData(i) = p {
                builder.with_payload(Payload::TaggedData(Box::new((i.as_ref()).try_into()?)))
            } else {
                return Err(Error::InvalidField("payload"));
            }
        } else {
            builder
        };

        builder.finish().map_err(Into::into)
    }
}

/// Describes all the different input types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputDto {
    Utxo(UtxoInputDto),
    Treasury(TreasuryInputDto),
}

impl From<&Input> for InputDto {
    fn from(value: &Input) -> Self {
        match value {
            Input::Utxo(u) => InputDto::Utxo(UtxoInputDto {
                kind: UtxoInput::KIND,
                transaction_id: u.output_id().transaction_id().to_string(),
                transaction_output_index: u.output_id().index(),
            }),
            Input::Treasury(t) => InputDto::Treasury(TreasuryInputDto {
                kind: TreasuryInput::KIND,
                milestone_id: t.milestone_id().to_string(),
            }),
        }
    }
}

impl TryFrom<&InputDto> for Input {
    type Error = Error;

    fn try_from(value: &InputDto) -> Result<Self, Self::Error> {
        match value {
            InputDto::Utxo(i) => Ok(Input::Utxo(UtxoInput::new(
                i.transaction_id
                    .parse::<TransactionId>()
                    .map_err(|_| Error::InvalidField("transactionId"))?,
                i.transaction_output_index,
            )?)),
            InputDto::Treasury(t) => Ok(Input::Treasury(
                t.milestone_id
                    .parse::<MilestoneId>()
                    .map_err(|_| Error::InvalidField("milestoneId"))?
                    .into(),
            )),
        }
    }
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
    Treasury(TreasuryOutputDto),
    Extended(ExtendedOutputDto),
    Alias(AliasOutputDto),
    Foundry(FoundryOutputDto),
    Nft(NftOutputDto),
}

impl From<&Output> for OutputDto {
    fn from(value: &Output) -> Self {
        match value {
            Output::Treasury(o) => OutputDto::Treasury(o.into()),
            Output::Extended(o) => OutputDto::Extended(o.into()),
            Output::Alias(o) => OutputDto::Alias(o.into()),
            Output::Foundry(o) => OutputDto::Foundry(o.into()),
            Output::Nft(o) => OutputDto::Nft(o.into()),
        }
    }
}

impl TryFrom<&OutputDto> for Output {
    type Error = Error;

    fn try_from(value: &OutputDto) -> Result<Self, Self::Error> {
        match value {
            OutputDto::Treasury(o) => Ok(Output::Treasury(o.try_into()?)),
            OutputDto::Extended(o) => Ok(Output::Extended(o.try_into()?)),
            OutputDto::Alias(o) => Ok(Output::Alias(o.try_into()?)),
            OutputDto::Foundry(o) => Ok(Output::Foundry(o.try_into()?)),
            OutputDto::Nft(o) => Ok(Output::Nft(o.try_into()?)),
        }
    }
}

impl<'de> serde::Deserialize<'de> for OutputDto {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        Ok(
            match value
                .get("type")
                .and_then(Value::as_u64)
                .ok_or_else(|| serde::de::Error::custom("invalid output type"))? as u8
            {
                TreasuryOutput::KIND => OutputDto::Treasury(
                    TreasuryOutputDto::deserialize(value)
                        .map_err(|e| serde::de::Error::custom(format!("cannot deserialize treasury output: {}", e)))?,
                ),
                ExtendedOutput::KIND => OutputDto::Extended(
                    ExtendedOutputDto::deserialize(value)
                        .map_err(|e| serde::de::Error::custom(format!("cannot deserialize extended output: {}", e)))?,
                ),
                AliasOutput::KIND => OutputDto::Alias(
                    AliasOutputDto::deserialize(value)
                        .map_err(|e| serde::de::Error::custom(format!("cannot deserialize alias output: {}", e)))?,
                ),
                FoundryOutput::KIND => OutputDto::Foundry(
                    FoundryOutputDto::deserialize(value)
                        .map_err(|e| serde::de::Error::custom(format!("cannot deserialize foundry output: {}", e)))?,
                ),
                NftOutput::KIND => OutputDto::Nft(
                    NftOutputDto::deserialize(value)
                        .map_err(|e| serde::de::Error::custom(format!("cannot deserialize NFT output: {}", e)))?,
                ),
                _ => return Err(serde::de::Error::custom("unsupported output type")),
            },
        )
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
            T1(&'a TreasuryOutputDto),
            T2(&'a ExtendedOutputDto),
            T3(&'a AliasOutputDto),
            T4(&'a FoundryOutputDto),
            T5(&'a NftOutputDto),
        }
        #[derive(Serialize)]
        struct TypedOutput<'a> {
            #[serde(flatten)]
            output: OutputDto_<'a>,
        }
        let output = match self {
            OutputDto::Treasury(o) => TypedOutput {
                output: OutputDto_::T1(o),
            },
            OutputDto::Extended(o) => TypedOutput {
                output: OutputDto_::T2(o),
            },
            OutputDto::Alias(o) => TypedOutput {
                output: OutputDto_::T3(o),
            },
            OutputDto::Foundry(o) => TypedOutput {
                output: OutputDto_::T4(o),
            },
            OutputDto::Nft(o) => TypedOutput {
                output: OutputDto_::T5(o),
            },
        };
        output.serialize(serializer)
    }
}

/// Describes all the different address types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddressDto {
    /// An Ed25519 address.
    Ed25519(Ed25519AddressDto),
    /// An alias address.
    Alias(AliasAddressDto),
    /// A NFT address.
    Nft(NftAddressDto),
}

impl From<&Address> for AddressDto {
    fn from(value: &Address) -> Self {
        match value {
            Address::Ed25519(a) => AddressDto::Ed25519(a.into()),
            Address::Alias(a) => AddressDto::Alias(a.into()),
            Address::Nft(a) => AddressDto::Nft(a.into()),
        }
    }
}

impl TryFrom<&AddressDto> for Address {
    type Error = Error;

    fn try_from(value: &AddressDto) -> Result<Self, Self::Error> {
        match value {
            AddressDto::Ed25519(a) => Ok(Address::Ed25519(a.try_into()?)),
            AddressDto::Alias(a) => Ok(Address::Alias(a.try_into()?)),
            AddressDto::Nft(a) => Ok(Address::Nft(a.try_into()?)),
        }
    }
}

/// Describes an Ed25519 address.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ed25519AddressDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub address: String,
}

impl From<&Ed25519Address> for Ed25519AddressDto {
    fn from(value: &Ed25519Address) -> Self {
        Self {
            kind: Ed25519Address::KIND,
            address: value.to_string(),
        }
    }
}

impl TryFrom<&Ed25519AddressDto> for Ed25519Address {
    type Error = Error;

    fn try_from(value: &Ed25519AddressDto) -> Result<Self, Self::Error> {
        value
            .address
            .parse::<Ed25519Address>()
            .map_err(|_| Error::InvalidField("Ed25519 address"))
    }
}

/// Describes an alias address.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AliasAddressDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub id: String,
}

impl From<&AliasAddress> for AliasAddressDto {
    fn from(value: &AliasAddress) -> Self {
        Self {
            kind: AliasAddress::KIND,
            id: value.to_string(),
        }
    }
}

impl TryFrom<&AliasAddressDto> for AliasAddress {
    type Error = Error;

    fn try_from(value: &AliasAddressDto) -> Result<Self, Self::Error> {
        value
            .id
            .parse::<AliasAddress>()
            .map_err(|_| Error::InvalidField("alias address"))
    }
}

/// Describes an NFT address.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NftAddressDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub id: String,
}

impl From<&NftAddress> for NftAddressDto {
    fn from(value: &NftAddress) -> Self {
        Self {
            kind: NftAddress::KIND,
            id: value.to_string(),
        }
    }
}

impl TryFrom<&NftAddressDto> for NftAddress {
    type Error = Error;

    fn try_from(value: &NftAddressDto) -> Result<Self, Self::Error> {
        value
            .id
            .parse::<NftAddress>()
            .map_err(|_| Error::InvalidField("NFT address"))
    }
}

/// Describes a treasury output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreasuryOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub amount: u64,
}

impl From<&TreasuryOutput> for TreasuryOutputDto {
    fn from(value: &TreasuryOutput) -> Self {
        Self {
            kind: TreasuryOutput::KIND,
            amount: value.amount(),
        }
    }
}

impl TryFrom<&TreasuryOutputDto> for TreasuryOutput {
    type Error = Error;

    fn try_from(value: &TreasuryOutputDto) -> Result<Self, Self::Error> {
        Ok(Self::new(value.amount)?)
    }
}

/// Describes all the different unlock types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnlockBlockDto {
    Signature(SignatureUnlockBlockDto),
    Reference(ReferenceUnlockBlockDto),
    Alias(AliasUnlockBlockDto),
    Nft(NftUnlockBlockDto),
}

impl From<&UnlockBlock> for UnlockBlockDto {
    fn from(value: &UnlockBlock) -> Self {
        match value {
            UnlockBlock::Signature(signature) => match signature.signature() {
                Signature::Ed25519(ed) => UnlockBlockDto::Signature(SignatureUnlockBlockDto {
                    kind: SignatureUnlockBlock::KIND,
                    signature: SignatureDto::Ed25519(Ed25519SignatureDto {
                        kind: Ed25519Signature::KIND,
                        public_key: hex::encode(ed.public_key()),
                        signature: hex::encode(ed.signature()),
                    }),
                }),
            },
            UnlockBlock::Reference(r) => UnlockBlockDto::Reference(ReferenceUnlockBlockDto {
                kind: ReferenceUnlockBlock::KIND,
                index: r.index(),
            }),
            UnlockBlock::Alias(a) => UnlockBlockDto::Alias(AliasUnlockBlockDto {
                kind: AliasUnlockBlock::KIND,
                index: a.index(),
            }),
            UnlockBlock::Nft(n) => UnlockBlockDto::Nft(NftUnlockBlockDto {
                kind: NftUnlockBlock::KIND,
                index: n.index(),
            }),
        }
    }
}

impl TryFrom<&UnlockBlockDto> for UnlockBlock {
    type Error = Error;

    fn try_from(value: &UnlockBlockDto) -> Result<Self, Self::Error> {
        match value {
            UnlockBlockDto::Signature(s) => match &s.signature {
                SignatureDto::Ed25519(ed) => {
                    let mut public_key = [0u8; Ed25519Address::LENGTH];
                    hex::decode_to_slice(&ed.public_key, &mut public_key)
                        .map_err(|_| Error::InvalidField("publicKey"))?;
                    // TODO const
                    let mut signature = [0u8; 64];
                    hex::decode_to_slice(&ed.signature, &mut signature)
                        .map_err(|_| Error::InvalidField("signature"))?;
                    Ok(UnlockBlock::Signature(SignatureUnlockBlock::from(Signature::Ed25519(
                        Ed25519Signature::new(public_key, signature),
                    ))))
                }
            },
            UnlockBlockDto::Reference(r) => Ok(UnlockBlock::Reference(ReferenceUnlockBlock::new(r.index)?)),
            UnlockBlockDto::Alias(a) => Ok(UnlockBlock::Alias(AliasUnlockBlock::new(a.index)?)),
            UnlockBlockDto::Nft(n) => Ok(UnlockBlock::Nft(NftUnlockBlock::new(n.index)?)),
        }
    }
}

/// Defines an unlock block containing signature(s) unlocking input(s).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureUnlockBlockDto {
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

/// References a previous unlock block in order to substitute the duplication of the same unlock block data for inputs
/// which unlock through the same data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferenceUnlockBlockDto {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "reference")]
    pub index: u16,
}

/// Points to the unlock block of a consumed alias output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AliasUnlockBlockDto {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "reference")]
    pub index: u16,
}

/// Points to the unlock block of a consumed NFT output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NftUnlockBlockDto {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "reference")]
    pub index: u16,
}

/// Describes an extended output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtendedOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    // Amount of IOTA tokens held by the output.
    pub amount: u64,
    // Native tokens held by the output.
    #[serde(rename = "nativeTokens")]
    pub native_tokens: Vec<NativeTokenDto>,
    #[serde(rename = "unlockConditions")]
    pub unlock_conditions: Vec<UnlockConditionDto>,
    #[serde(rename = "blocks")]
    pub feature_blocks: Vec<FeatureBlockDto>,
}

impl From<&ExtendedOutput> for ExtendedOutputDto {
    fn from(value: &ExtendedOutput) -> Self {
        Self {
            kind: ExtendedOutput::KIND,
            amount: value.amount(),
            native_tokens: value.native_tokens().iter().map(Into::into).collect::<_>(),
            unlock_conditions: value.unlock_conditions().iter().map(Into::into).collect::<_>(),
            feature_blocks: value.feature_blocks().iter().map(Into::into).collect::<_>(),
        }
    }
}

impl TryFrom<&ExtendedOutputDto> for ExtendedOutput {
    type Error = Error;

    fn try_from(value: &ExtendedOutputDto) -> Result<Self, Self::Error> {
        let mut builder = ExtendedOutputBuilder::new(value.amount);
        for t in &value.native_tokens {
            builder = builder.add_native_token(t.try_into()?);
        }
        for b in &value.unlock_conditions {
            builder = builder.add_unlock_condition(b.try_into()?);
        }
        for b in &value.feature_blocks {
            builder = builder.add_feature_block(b.try_into()?);
        }
        Ok(builder.finish()?)
    }
}

/// Describes a native token.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeTokenDto {
    // Identifier of the native token.
    pub token_id: TokenIdDto,
    // Amount of native tokens.
    pub amount: U256Dto,
}

impl From<&NativeToken> for NativeTokenDto {
    fn from(value: &NativeToken) -> Self {
        Self {
            token_id: TokenIdDto(value.token_id().to_string()),
            amount: U256Dto(value.amount().to_string()),
        }
    }
}

impl TryFrom<&NativeTokenDto> for NativeToken {
    type Error = Error;

    fn try_from(value: &NativeTokenDto) -> Result<Self, Self::Error> {
        Ok(Self::new(
            (&value.token_id).try_into()?,
            value
                .amount
                .0
                .parse::<U256>()
                .map_err(|_| Error::InvalidField("amount"))?,
        ))
    }
}

/// Describes a token id.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenIdDto(pub String);

impl From<&TokenId> for TokenIdDto {
    fn from(value: &TokenId) -> Self {
        Self(value.to_string())
    }
}

impl TryFrom<&TokenIdDto> for TokenId {
    type Error = Error;

    fn try_from(value: &TokenIdDto) -> Result<Self, Self::Error> {
        value.0.parse::<TokenId>().map_err(|_| Error::InvalidField("token id"))
    }
}

/// Describes an U256.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct U256Dto(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnlockConditionDto {
    /// An address unlock condition.
    Address(AddressUnlockConditionDto),
    /// A dust deposit return unlock condition.
    DustDepositReturn(DustDepositReturnUnlockConditionDto),
    /// A timelock unlock condition.
    Timelock(TimelockUnlockConditionDto),
    /// An expiration unlock condition.
    Expiration(ExpirationUnlockConditionDto),
    /// A state controller address unlock condition.
    StateControllerAddress(StateControllerAddressUnlockConditionDto),
    /// A governor address unlock condition.
    GovernorAddress(GovernorAddressUnlockConditionDto),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FeatureBlockDto {
    /// A sender feature block.
    Sender(SenderFeatureBlockDto),
    /// An issuer feature block.
    Issuer(IssuerFeatureBlockDto),
    /// A metadata feature block.
    Metadata(MetadataFeatureBlockDto),
    /// A tag feature block.
    Tag(TagFeatureBlockDto),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddressUnlockConditionDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub address: AddressDto,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DustDepositReturnUnlockConditionDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub return_address: Address,
    pub amount: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelockUnlockConditionDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub index: MilestoneIndex,
    pub timestamp: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpirationUnlockConditionDto {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "returnAddress")]
    pub return_address: AddressDto,
    pub index: MilestoneIndex,
    pub timestamp: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateControllerAddressUnlockConditionDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub address: AddressDto,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernorAddressUnlockConditionDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub address: AddressDto,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SenderFeatureBlockDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub address: AddressDto,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssuerFeatureBlockDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub address: AddressDto,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetadataFeatureBlockDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub data: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagFeatureBlockDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub tag: String,
}

impl UnlockConditionDto {
    /// Return the unlock condition kind of a `UnlockConditionDto`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Address(_) => AddressUnlockCondition::KIND,
            Self::DustDepositReturn(_) => DustDepositReturnUnlockCondition::KIND,
            Self::Timelock(_) => TimelockUnlockCondition::KIND,
            Self::Expiration(_) => ExpirationUnlockCondition::KIND,
            Self::StateControllerAddress(_) => StateControllerAddressUnlockCondition::KIND,
            Self::GovernorAddress(_) => GovernorAddressUnlockCondition::KIND,
        }
    }
}

impl FeatureBlockDto {
    /// Return the feature block kind of a `FeatureBlockDto`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Sender(_) => SenderFeatureBlock::KIND,
            Self::Issuer(_) => IssuerFeatureBlock::KIND,
            Self::Metadata(_) => MetadataFeatureBlock::KIND,
            Self::Tag(_) => TagFeatureBlock::KIND,
        }
    }
}

impl From<&UnlockCondition> for UnlockConditionDto {
    fn from(value: &UnlockCondition) -> Self {
        match value {
            UnlockCondition::Address(v) => Self::Address(AddressUnlockConditionDto {
                kind: AddressUnlockCondition::KIND,
                address: v.address().into(),
            }),
            UnlockCondition::DustDepositReturn(v) => Self::DustDepositReturn(DustDepositReturnUnlockConditionDto {
                kind: DustDepositReturnUnlockCondition::KIND,
                return_address: *v.return_address(),
                amount: v.amount(),
            }),
            UnlockCondition::Timelock(v) => Self::Timelock(TimelockUnlockConditionDto {
                kind: TimelockUnlockCondition::KIND,
                index: v.index(),
                timestamp: v.timestamp(),
            }),
            UnlockCondition::Expiration(v) => Self::Expiration(ExpirationUnlockConditionDto {
                kind: ExpirationUnlockCondition::KIND,
                return_address: v.return_address().into(),
                index: v.index(),
                timestamp: v.timestamp(),
            }),
            UnlockCondition::StateControllerAddress(v) => {
                Self::StateControllerAddress(StateControllerAddressUnlockConditionDto {
                    kind: StateControllerAddressUnlockCondition::KIND,
                    address: v.address().into(),
                })
            }
            UnlockCondition::GovernorAddress(v) => Self::GovernorAddress(GovernorAddressUnlockConditionDto {
                kind: GovernorAddressUnlockCondition::KIND,
                address: v.address().into(),
            }),
        }
    }
}

impl From<&FeatureBlock> for FeatureBlockDto {
    fn from(value: &FeatureBlock) -> Self {
        match value {
            FeatureBlock::Sender(v) => Self::Sender(SenderFeatureBlockDto {
                kind: SenderFeatureBlock::KIND,
                address: v.address().into(),
            }),
            FeatureBlock::Issuer(v) => Self::Issuer(IssuerFeatureBlockDto {
                kind: IssuerFeatureBlock::KIND,
                address: v.address().into(),
            }),
            FeatureBlock::Metadata(v) => Self::Metadata(MetadataFeatureBlockDto {
                kind: MetadataFeatureBlock::KIND,
                data: v.to_string(),
            }),
            FeatureBlock::Tag(v) => Self::Tag(TagFeatureBlockDto {
                kind: TagFeatureBlock::KIND,
                tag: v.to_string(),
            }),
        }
    }
}

impl TryFrom<&UnlockConditionDto> for UnlockCondition {
    type Error = Error;

    fn try_from(value: &UnlockConditionDto) -> Result<Self, Self::Error> {
        Ok(match value {
            UnlockConditionDto::Address(v) => Self::Address(AddressUnlockCondition::new(
                (&v.address)
                    .try_into()
                    .map_err(|_e| Error::InvalidField("AddressUnlockCondition"))?,
            )),
            UnlockConditionDto::DustDepositReturn(v) => {
                Self::DustDepositReturn(DustDepositReturnUnlockCondition::new(v.return_address, v.amount)?)
            }
            UnlockConditionDto::Timelock(v) => Self::Timelock(TimelockUnlockCondition::new(v.index, v.timestamp)),
            UnlockConditionDto::Expiration(v) => Self::Expiration(ExpirationUnlockCondition::new(
                (&v.return_address)
                    .try_into()
                    .map_err(|_e| Error::InvalidField("ExpirationUnlockCondition"))?,
                v.index,
                v.timestamp,
            )),
            UnlockConditionDto::StateControllerAddress(v) => {
                Self::StateControllerAddress(StateControllerAddressUnlockCondition::new(
                    (&v.address)
                        .try_into()
                        .map_err(|_e| Error::InvalidField("StateControllerAddressUnlockCondition"))?,
                ))
            }
            UnlockConditionDto::GovernorAddress(v) => Self::GovernorAddress(GovernorAddressUnlockCondition::new(
                (&v.address)
                    .try_into()
                    .map_err(|_e| Error::InvalidField("GovernorAddressUnlockCondition"))?,
            )),
        })
    }
}

impl TryFrom<&FeatureBlockDto> for FeatureBlock {
    type Error = Error;

    fn try_from(value: &FeatureBlockDto) -> Result<Self, Self::Error> {
        Ok(match value {
            FeatureBlockDto::Sender(v) => Self::Sender(SenderFeatureBlock::new((&v.address).try_into()?)),
            FeatureBlockDto::Issuer(v) => Self::Issuer(IssuerFeatureBlock::new((&v.address).try_into()?)),
            FeatureBlockDto::Metadata(v) => Self::Metadata(MetadataFeatureBlock::new(
                hex::decode(&v.data).map_err(|_e| Error::InvalidField("MetadataFeatureBlock"))?,
            )?),
            FeatureBlockDto::Tag(v) => Self::Tag(TagFeatureBlock::new(
                hex::decode(&v.tag).map_err(|_e| Error::InvalidField("TagFeatureBlock"))?,
            )?),
        })
    }
}

/// Describes an alias account in the ledger that can be controlled by the state and governance controllers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AliasOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    // Amount of IOTA tokens held by the output.
    pub amount: u64,
    // Native tokens held by the output.
    #[serde(rename = "nativeTokens")]
    pub native_tokens: Vec<NativeTokenDto>,
    // Unique identifier of the alias.
    #[serde(rename = "aliasId")]
    pub alias_id: AliasIdDto,
    // A counter that must increase by 1 every time the alias is state transitioned.
    #[serde(rename = "stateIndex")]
    pub state_index: u32,
    // Metadata that can only be changed by the state controller.
    #[serde(rename = "stateMetadata")]
    pub state_metadata: String,
    // A counter that denotes the number of foundries created by this alias account.
    #[serde(rename = "foundryCounter")]
    pub foundry_counter: u32,
    //
    #[serde(rename = "unlockConditions")]
    pub unlock_conditions: Vec<UnlockConditionDto>,
    //
    #[serde(rename = "blocks")]
    pub feature_blocks: Vec<FeatureBlockDto>,
}

impl From<&AliasOutput> for AliasOutputDto {
    fn from(value: &AliasOutput) -> Self {
        Self {
            kind: AliasOutput::KIND,
            amount: value.amount(),
            native_tokens: value.native_tokens().iter().map(Into::into).collect::<_>(),
            alias_id: AliasIdDto(value.alias_id().to_string()),
            state_index: value.state_index(),
            state_metadata: hex::encode(value.state_metadata()),
            foundry_counter: value.foundry_counter(),
            unlock_conditions: value.unlock_conditions().iter().map(Into::into).collect::<_>(),
            feature_blocks: value.feature_blocks().iter().map(Into::into).collect::<_>(),
        }
    }
}

impl TryFrom<&AliasOutputDto> for AliasOutput {
    type Error = Error;

    fn try_from(value: &AliasOutputDto) -> Result<Self, Self::Error> {
        let mut builder = AliasOutputBuilder::new(value.amount, (&value.alias_id).try_into()?)?;
        builder = builder.with_state_index(value.state_index);
        builder = builder.with_state_metadata(
            hex::decode(&value.state_metadata).map_err(|_| Error::InvalidField("state_metadata"))?,
        );
        builder = builder.with_foundry_counter(value.foundry_counter);

        for t in &value.native_tokens {
            builder = builder.add_native_token(t.try_into()?);
        }
        for b in &value.unlock_conditions {
            builder = builder.add_unlock_condition(b.try_into()?);
        }
        for b in &value.feature_blocks {
            builder = builder.add_feature_block(b.try_into()?);
        }
        Ok(builder.finish()?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AliasIdDto(pub String);

impl From<&AliasId> for AliasIdDto {
    fn from(value: &AliasId) -> Self {
        Self(value.to_string())
    }
}

impl TryFrom<&AliasIdDto> for AliasId {
    type Error = Error;

    fn try_from(value: &AliasIdDto) -> Result<Self, Self::Error> {
        value.0.parse::<AliasId>().map_err(|_| Error::InvalidField("alias id"))
    }
}

/// Describes a foundry output that is controlled by an alias.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FoundryOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    // Amount of IOTA tokens held by the output.
    amount: u64,
    // Native tokens held by the output.
    #[serde(rename = "nativeTokens")]
    native_tokens: Vec<NativeTokenDto>,
    // The serial number of the foundry with respect to the controlling alias.
    #[serde(rename = "serialNumber")]
    serial_number: u32,
    // Data that is always the last 12 bytes of ID of the tokens produced by this foundry.
    #[serde(rename = "tokenTag")]
    token_tag: String,
    // Circulating supply of tokens controlled by this foundry.
    #[serde(rename = "circulatingSupply")]
    circulating_supply: U256Dto,
    // Maximum supply of tokens controlled by this foundry.
    #[serde(rename = "maximumSupply")]
    maximum_supply: U256Dto,
    #[serde(rename = "tokenScheme")]
    token_scheme: TokenSchemeDto,
    #[serde(rename = "unlockConditions")]
    unlock_conditions: Vec<UnlockConditionDto>,
    #[serde(rename = "blocks")]
    feature_blocks: Vec<FeatureBlockDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TokenSchemeDto {
    Simple = 0,
}

impl From<&FoundryOutput> for FoundryOutputDto {
    fn from(value: &FoundryOutput) -> Self {
        Self {
            kind: FoundryOutput::KIND,
            amount: value.amount(),
            native_tokens: value.native_tokens().iter().map(Into::into).collect::<_>(),
            serial_number: value.serial_number(),
            token_tag: hex::encode(value.token_tag()),
            circulating_supply: U256Dto(value.circulating_supply().to_string()),
            maximum_supply: U256Dto(value.maximum_supply().to_string()),
            token_scheme: match value.token_scheme() {
                TokenScheme::Simple => TokenSchemeDto::Simple,
            },
            unlock_conditions: value.unlock_conditions().iter().map(Into::into).collect::<_>(),
            feature_blocks: value.feature_blocks().iter().map(Into::into).collect::<_>(),
        }
    }
}

impl TryFrom<&FoundryOutputDto> for FoundryOutput {
    type Error = Error;

    fn try_from(value: &FoundryOutputDto) -> Result<Self, Self::Error> {
        let mut builder = FoundryOutputBuilder::new(
            value.amount,
            value.serial_number,
            {
                let mut decoded_token_tag = [0u8; 12];
                hex::decode_to_slice(&value.token_tag, &mut decoded_token_tag as &mut [u8])
                    .map_err(|_| Error::InvalidField("token_tag"))?;
                decoded_token_tag
            },
            value
                .circulating_supply
                .0
                .parse::<U256>()
                .map_err(|_| Error::InvalidField("circulating_supply"))?,
            value
                .maximum_supply
                .0
                .parse::<U256>()
                .map_err(|_| Error::InvalidField("maximum_supply"))?,
            match value.token_scheme {
                TokenSchemeDto::Simple => TokenScheme::Simple,
            },
        )?;

        for t in &value.native_tokens {
            builder = builder.add_native_token(t.try_into()?);
        }
        for b in &value.unlock_conditions {
            builder = builder.add_unlock_condition(b.try_into()?);
        }
        for b in &value.feature_blocks {
            builder = builder.add_feature_block(b.try_into()?);
        }

        Ok(builder.finish()?)
    }
}

/// Describes an NFT output, a globally unique token with metadata attached.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NftOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    // Amount of IOTA tokens held by the output.
    pub amount: u64,
    // Native tokens held by the output.
    #[serde(rename = "nativeTokens")]
    pub native_tokens: Vec<NativeTokenDto>,
    // Unique identifier of the NFT.
    #[serde(rename = "nftId")]
    pub nft_id: NftIdDto,
    // Binary metadata attached immutably to the NFT.
    #[serde(rename = "immutableData")]
    pub immutable_data: String,
    #[serde(rename = "unlockConditions")]
    pub unlock_conditions: Vec<UnlockConditionDto>,
    #[serde(rename = "blocks")]
    pub feature_blocks: Vec<FeatureBlockDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NftIdDto(pub String);

impl From<&NftId> for NftIdDto {
    fn from(value: &NftId) -> Self {
        Self(value.to_string())
    }
}

impl TryFrom<&NftIdDto> for NftId {
    type Error = Error;

    fn try_from(value: &NftIdDto) -> Result<Self, Self::Error> {
        value.0.parse::<NftId>().map_err(|_| Error::InvalidField("NFT id"))
    }
}

impl From<&NftOutput> for NftOutputDto {
    fn from(value: &NftOutput) -> Self {
        Self {
            kind: NftOutput::KIND,
            amount: value.amount(),
            native_tokens: value.native_tokens().iter().map(Into::into).collect::<_>(),
            nft_id: NftIdDto(value.nft_id().to_string()),
            immutable_data: hex::encode(&value.immutable_metadata()),
            unlock_conditions: value.unlock_conditions().iter().map(Into::into).collect::<_>(),
            feature_blocks: value.feature_blocks().iter().map(Into::into).collect::<_>(),
        }
    }
}

impl TryFrom<&NftOutputDto> for NftOutput {
    type Error = Error;

    fn try_from(value: &NftOutputDto) -> Result<Self, Self::Error> {
        let mut builder = NftOutputBuilder::new(
            value.amount,
            (&value.nft_id).try_into()?,
            hex::decode(&value.immutable_data).map_err(|_| Error::InvalidField("immutableData"))?,
        )?;

        for t in &value.native_tokens {
            builder = builder.add_native_token(t.try_into()?);
        }
        for b in &value.unlock_conditions {
            builder = builder.add_unlock_condition(b.try_into()?);
        }
        for b in &value.feature_blocks {
            builder = builder.add_feature_block(b.try_into()?);
        }

        Ok(builder.finish()?)
    }
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

impl From<&MilestonePayload> for MilestonePayloadDto {
    fn from(value: &MilestonePayload) -> Self {
        MilestonePayloadDto {
            kind: MilestonePayload::KIND,
            index: *value.essence().index(),
            timestamp: value.essence().timestamp(),
            parents: value.essence().parents().iter().map(|p| p.to_string()).collect(),
            inclusion_merkle_proof: hex::encode(value.essence().merkle_proof()),
            next_pow_score: value.essence().next_pow_score(),
            next_pow_score_milestone_index: value.essence().next_pow_score_milestone_index(),
            public_keys: value.essence().public_keys().iter().map(hex::encode).collect(),
            receipt: value.essence().receipt().map(Into::into),
            signatures: value.signatures().map(hex::encode).collect(),
        }
    }
}

impl TryFrom<&MilestonePayloadDto> for MilestonePayload {
    type Error = Error;

    fn try_from(value: &MilestonePayloadDto) -> Result<Self, Self::Error> {
        let essence = {
            let index = value.index;
            let timestamp = value.timestamp;
            let mut parent_ids = Vec::new();
            for msg_id in &value.parents {
                parent_ids.push(
                    msg_id
                        .parse::<MessageId>()
                        .map_err(|_| Error::InvalidField("parentMessageIds"))?,
                );
            }
            let merkle_proof = {
                let mut buf = [0u8; MilestoneEssence::MERKLE_PROOF_LENGTH];
                hex::decode_to_slice(&value.inclusion_merkle_proof, &mut buf)
                    .map_err(|_| Error::InvalidField("inclusionMerkleProof"))?;
                buf
            };
            let next_pow_score = value.next_pow_score;
            let next_pow_score_milestone_index = value.next_pow_score_milestone_index;
            let mut public_keys = Vec::new();
            for v in &value.public_keys {
                let key = {
                    let mut buf = [0u8; MilestoneEssence::PUBLIC_KEY_LENGTH];
                    hex::decode_to_slice(v, &mut buf).map_err(|_| Error::InvalidField("publicKeys"))?;
                    buf
                };
                public_keys.push(key);
            }
            let receipt = if let Some(receipt) = value.receipt.as_ref() {
                Some(receipt.try_into()?)
            } else {
                None
            };
            MilestoneEssence::new(
                MilestoneIndex(index),
                timestamp,
                Parents::new(parent_ids)?,
                merkle_proof,
                next_pow_score,
                next_pow_score_milestone_index,
                public_keys,
                receipt,
            )?
        };
        let mut signatures = Vec::new();
        for v in &value.signatures {
            signatures.push(
                hex::decode(v)
                    .map_err(|_| Error::InvalidField("signatures"))?
                    .try_into()
                    .map_err(|_| Error::InvalidField("signatures"))?,
            )
        }

        Ok(MilestonePayload::new(essence, signatures)?)
    }
}

/// The payload type to define a tagged data payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaggedDataPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub tag: String,
    pub data: String,
}

impl From<&TaggedDataPayload> for TaggedDataPayloadDto {
    fn from(value: &TaggedDataPayload) -> Self {
        TaggedDataPayloadDto {
            kind: TaggedDataPayload::KIND,
            tag: hex::encode(value.tag()),
            data: hex::encode(value.data()),
        }
    }
}

impl TryFrom<&TaggedDataPayloadDto> for TaggedDataPayload {
    type Error = Error;

    fn try_from(value: &TaggedDataPayloadDto) -> Result<Self, Self::Error> {
        Ok(TaggedDataPayload::new(
            hex::decode(value.tag.clone()).map_err(|_| Error::InvalidField("index"))?,
            hex::decode(value.data.clone()).map_err(|_| Error::InvalidField("data"))?,
        )?)
    }
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

impl From<&ReceiptPayload> for ReceiptPayloadDto {
    fn from(value: &ReceiptPayload) -> Self {
        ReceiptPayloadDto {
            kind: ReceiptPayload::KIND,
            migrated_at: *value.migrated_at(),
            last: value.last(),
            funds: value.funds().iter().map(Into::into).collect::<_>(),
            transaction: value.transaction().into(),
        }
    }
}

impl TryFrom<&ReceiptPayloadDto> for ReceiptPayload {
    type Error = Error;

    fn try_from(value: &ReceiptPayloadDto) -> Result<Self, Self::Error> {
        Ok(ReceiptPayload::new(
            MilestoneIndex(value.migrated_at),
            value.last,
            value.funds.iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            (&value.transaction).try_into()?,
        )?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigratedFundsEntryDto {
    #[serde(rename = "tailTransactionHash")]
    pub tail_transaction_hash: String,
    pub address: AddressDto,
    pub deposit: u64,
}

impl From<&MigratedFundsEntry> for MigratedFundsEntryDto {
    fn from(value: &MigratedFundsEntry) -> Self {
        MigratedFundsEntryDto {
            tail_transaction_hash: hex::encode(value.tail_transaction_hash().as_ref()),
            address: value.address().into(),
            deposit: value.amount(),
        }
    }
}

impl TryFrom<&MigratedFundsEntryDto> for MigratedFundsEntry {
    type Error = Error;

    fn try_from(value: &MigratedFundsEntryDto) -> Result<Self, Self::Error> {
        let mut tail_transaction_hash = [0u8; TailTransactionHash::LENGTH];
        hex::decode_to_slice(&value.tail_transaction_hash, &mut tail_transaction_hash)
            .map_err(|_| Error::InvalidField("tailTransactionHash"))?;
        Ok(MigratedFundsEntry::new(
            TailTransactionHash::new(tail_transaction_hash)?,
            (&value.address).try_into()?,
            value.deposit,
        )?)
    }
}

/// The payload type to define a treasury transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreasuryTransactionPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub input: InputDto,
    pub output: OutputDto,
}

impl From<&TreasuryTransactionPayload> for TreasuryTransactionPayloadDto {
    fn from(value: &TreasuryTransactionPayload) -> Self {
        TreasuryTransactionPayloadDto {
            kind: TreasuryTransactionPayload::KIND,
            input: value.input().into(),
            output: value.output().into(),
        }
    }
}

impl TryFrom<&TreasuryTransactionPayloadDto> for TreasuryTransactionPayload {
    type Error = Error;

    fn try_from(value: &TreasuryTransactionPayloadDto) -> Result<Self, Self::Error> {
        Ok(TreasuryTransactionPayload::new(
            Input::try_from(&value.input)?,
            Output::try_from(&value.output)?,
        )?)
    }
}

/// Describes a peer.
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

#[cfg(feature = "peer")]
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
                    RelationDto::Autopeered
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

/// Returns all information about the gossip stream with the peer.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GossipDto {
    pub heartbeat: HeartbeatDto,
    pub metrics: MetricsDto,
}

/// Describes the relation with the peer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RelationDto {
    #[serde(rename = "known")]
    Known,
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "autopeered")]
    Autopeered,
}

/// Describes the heartbeat of a node.
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

/// Describes metrics of a gossip stream.
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

/// Describes a receipt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiptDto {
    pub receipt: ReceiptPayloadDto,
    #[serde(rename = "milestoneIndex")]
    pub milestone_index: u32,
}

impl From<Receipt> for ReceiptDto {
    fn from(value: Receipt) -> Self {
        ReceiptDto {
            receipt: value.inner().into(),
            milestone_index: **value.included_in(),
        }
    }
}

/// Describes the ledger inclusion state of a transaction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LedgerInclusionStateDto {
    #[serde(rename = "conflicting")]
    Conflicting,
    #[serde(rename = "included")]
    Included,
    #[serde(rename = "noTransaction")]
    NoTransaction,
}
