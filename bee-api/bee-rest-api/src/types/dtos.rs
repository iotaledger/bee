// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::error::Error;

use bee_ledger::types::Receipt;
pub use bee_message::output::feature_block::{
    DustDepositReturnFeatureBlock, ExpirationMilestoneIndexFeatureBlock, ExpirationUnixFeatureBlock,
    IndexationFeatureBlock, IssuerFeatureBlock, MetadataFeatureBlock, SenderFeatureBlock,
    TimelockMilestoneIndexFeatureBlock, TimelockUnixFeatureBlock,
};
use bee_message::{
    address::{Address, AliasAddress, Ed25519Address, NftAddress},
    input::{Input, TreasuryInput, UtxoInput},
    milestone::MilestoneIndex,
    output::{
        feature_block::FeatureBlock, ExtendedOutput, ExtendedOutputBuilder, NativeToken, Output, SimpleOutput, TokenId,
        TreasuryOutput,
    },
    parent::Parents,
    payload::{
        indexation::IndexationPayload,
        milestone::{MilestoneEssence, MilestoneId, MilestonePayload},
        receipt::{MigratedFundsEntry, ReceiptPayload, TailTransactionHash},
        transaction::{RegularTransactionEssence, TransactionEssence, TransactionId, TransactionPayload},
        treasury::TreasuryTransactionPayload,
        Payload,
    },
    signature::{Ed25519Signature, Signature},
    unlock_block::{ReferenceUnlockBlock, SignatureUnlockBlock, UnlockBlock, UnlockBlocks},
    Message, MessageBuilder, MessageId,
};

#[cfg(feature = "peer")]
use bee_protocol::types::peer::Peer;

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
                    .map_err(|_| Error::InvalidSyntaxField("networkId"))?,
            )
            .with_parents(Parents::new(
                value
                    .parents
                    .iter()
                    .map(|m| {
                        m.parse::<MessageId>()
                            .map_err(|_| Error::InvalidSyntaxField("parentMessageIds"))
                    })
                    .collect::<Result<Vec<MessageId>, Error>>()?,
            )?)
            .with_nonce_provider(
                value
                    .nonce
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidSyntaxField("nonce"))?,
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
    Indexation(Box<IndexationPayloadDto>),
    Receipt(Box<ReceiptPayloadDto>),
    TreasuryTransaction(Box<TreasuryTransactionPayloadDto>),
}

impl From<&Payload> for PayloadDto {
    fn from(value: &Payload) -> Self {
        match value {
            Payload::Transaction(t) => PayloadDto::Transaction(Box::new(TransactionPayloadDto::from(t.as_ref()))),
            Payload::Milestone(m) => PayloadDto::Milestone(Box::new(MilestonePayloadDto::from(m.as_ref()))),
            Payload::Indexation(i) => PayloadDto::Indexation(Box::new(IndexationPayloadDto::from(i.as_ref()))),
            Payload::Receipt(r) => PayloadDto::Receipt(Box::new(ReceiptPayloadDto::from(r.as_ref()))),
            Payload::TreasuryTransaction(t) => {
                PayloadDto::TreasuryTransaction(Box::new(TreasuryTransactionPayloadDto::from(t.as_ref())))
            }
        }
    }
}

impl TryFrom<&PayloadDto> for Payload {
    type Error = Error;
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
            unlock_blocks: value.unlock_blocks().iter().map(|u| u.into()).collect::<Vec<_>>(),
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
            inputs: value.inputs().iter().map(|i| i.into()).collect::<Vec<_>>(),
            outputs: value.outputs().iter().map(|o| o.into()).collect::<Vec<_>>(),
            payload: match value.payload() {
                Some(Payload::Indexation(i)) => Some(PayloadDto::Indexation(Box::new(i.as_ref().into()))),
                Some(_) => unimplemented!(),
                None => None,
            },
        }
    }
}

impl TryFrom<&RegularTransactionEssenceDto> for RegularTransactionEssence {
    type Error = Error;

    fn try_from(value: &RegularTransactionEssenceDto) -> Result<Self, Self::Error> {
        let mut builder = RegularTransactionEssence::builder();

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
                return Err(Error::InvalidSemanticField("payload"));
            }
        }

        Ok(builder.finish()?)
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
                    .map_err(|_| Error::InvalidSyntaxField("transactionId"))?,
                i.transaction_output_index,
            )?)),
            InputDto::Treasury(t) => Ok(Input::Treasury(
                t.milestone_id
                    .parse::<MilestoneId>()
                    .map_err(|_| Error::InvalidSyntaxField("milestoneId"))?
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
    Simple(SimpleOutputDto),
    Treasury(TreasuryOutputDto),
    Extended(ExtendedOutputDto),
}

impl From<&Output> for OutputDto {
    fn from(value: &Output) -> Self {
        match value {
            Output::Simple(s) => OutputDto::Simple(SimpleOutputDto {
                kind: SimpleOutput::KIND,
                address: s.address().into(),
                amount: s.amount(),
            }),
            Output::Treasury(t) => OutputDto::Treasury(TreasuryOutputDto {
                kind: TreasuryOutput::KIND,
                amount: t.amount(),
            }),
            Output::Extended(e) => OutputDto::Extended(ExtendedOutputDto {
                address: e.address().into(),
                amount: e.amount(),
                native_tokens: e.native_tokens().iter().map(|o| o.into()).collect(),
                feature_blocks: e.feature_blocks().iter().map(|o| o.into()).collect(),
            }),
            Output::Alias(_) => todo!(),
            Output::Foundry(_) => todo!(),
            Output::Nft(_) => todo!(),
        }
    }
}

impl TryFrom<&OutputDto> for Output {
    type Error = Error;

    fn try_from(value: &OutputDto) -> Result<Self, Self::Error> {
        match value {
            OutputDto::Simple(s) => Ok(Output::Simple(SimpleOutput::new((&s.address).try_into()?, s.amount)?)),
            OutputDto::Treasury(t) => Ok(Output::Treasury(TreasuryOutput::new(t.amount)?)),
            OutputDto::Extended(e) => Ok(Output::Extended(e.try_into()?)),
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
                SimpleOutput::KIND => OutputDto::Simple(
                    SimpleOutputDto::deserialize(value)
                        .map_err(|e| serde::de::Error::custom(format!("can not deserialize output: {}", e)))?,
                ),
                TreasuryOutput::KIND => OutputDto::Treasury(
                    TreasuryOutputDto::deserialize(value)
                        .map_err(|e| serde::de::Error::custom(format!("can not deserialize output: {}", e)))?,
                ),
                ExtendedOutput::KIND => OutputDto::Extended(
                    ExtendedOutputDto::deserialize(value)
                        .map_err(|e| serde::de::Error::custom(format!("can not deserialize output: {}", e)))?,
                ),
                _ => unimplemented!(),
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
            T1(&'a SimpleOutputDto),
            T2(&'a TreasuryOutputDto),
            T3(&'a ExtendedOutputDto),
        }
        #[derive(Serialize)]
        struct TypedOutput<'a> {
            #[serde(flatten)]
            output: OutputDto_<'a>,
        }
        let output = match self {
            OutputDto::Simple(o) => TypedOutput {
                output: OutputDto_::T1(o),
            },
            OutputDto::Treasury(o) => TypedOutput {
                output: OutputDto_::T2(o),
            },
            OutputDto::Extended(o) => TypedOutput {
                output: OutputDto_::T3(o),
            },
        };
        output.serialize(serializer)
    }
}

/// Describes a deposit to a single address which is unlocked via a signature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimpleOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub address: AddressDto,
    pub amount: u64,
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
            .map_err(|_| Error::InvalidSyntaxField("address"))
    }
}

/// Describes an Ed25519 address.
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
            .map_err(|_| Error::InvalidSyntaxField("address"))
    }
}

/// Describes an Ed25519 address.
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
            .map_err(|_| Error::InvalidSyntaxField("address"))
    }
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
    Signature(SignatureUnlockBlockDto),
    Reference(ReferenceUnlockBlockDto),
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
            UnlockBlock::Alias(_) => todo!(),
            UnlockBlock::Nft(_) => todo!(),
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
                        .map_err(|_| Error::InvalidSyntaxField("publicKey"))?;
                    // TODO const
                    let mut signature = [0u8; 64];
                    hex::decode_to_slice(&ed.signature, &mut signature)
                        .map_err(|_| Error::InvalidSyntaxField("signature"))?;
                    Ok(UnlockBlock::Signature(SignatureUnlockBlock::from(Signature::Ed25519(
                        Ed25519Signature::new(public_key, signature),
                    ))))
                }
            },
            UnlockBlockDto::Reference(r) => Ok(UnlockBlock::Reference(ReferenceUnlockBlock::new(r.index)?)),
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

/// Describes an extended output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtendedOutputDto {
    // Deposit address of the output.
    pub address: AddressDto,
    // Amount of IOTA tokens held by the output.
    pub amount: u64,
    // Native tokens held by the output.
    pub native_tokens: Vec<NativeTokenDto>,
    pub feature_blocks: Vec<FeatureBlockDto>,
}

impl From<&ExtendedOutput> for ExtendedOutputDto {
    fn from(value: &ExtendedOutput) -> Self {
        Self {
            address: value.address().into(),
            amount: value.amount(),
            native_tokens: value.native_tokens().iter().map(|x| x.into()).collect::<_>(),
            feature_blocks: value.feature_blocks().iter().map(|x| x.into()).collect::<_>(),
        }
    }
}

impl TryFrom<&ExtendedOutputDto> for ExtendedOutput {
    type Error = Error;

    fn try_from(value: &ExtendedOutputDto) -> Result<Self, Self::Error> {
        let mut builder = ExtendedOutputBuilder::new((&value.address).try_into()?, value.amount);
        for t in &value.native_tokens {
            builder = builder.add_native_token(t.try_into()?);
        }
        for b in &value.feature_blocks {
            builder = builder.add_feature_block(b.try_into()?);
        }
        Ok(builder.finish()?)
    }
}

/// Describes a native token.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
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
        value.try_into()
    }
}

/// Describes a token id.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TokenIdDto(pub String);

impl From<&TokenId> for TokenIdDto {
    fn from(value: &TokenId) -> Self {
        Self(value.to_string())
    }
}

impl TryFrom<&TokenIdDto> for TokenId {
    type Error = Error;

    fn try_from(value: &TokenIdDto) -> Result<Self, Self::Error> {
        value
            .0
            .parse::<TokenId>()
            .map_err(|_| Error::InvalidSemanticField("token id"))
    }
}

/// Describes an U256.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct U256Dto(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FeatureBlockDto {
    /// A sender feature block.
    Sender(SenderFeatureBlockDto),
    /// An issuer feature block.
    Issuer(IssuerFeatureBlockDto),
    /// A dust deposit return feature block.
    DustDepositReturn(DustDepositReturnFeatureBlockDto),
    /// A timelock milestone index feature block.
    TimelockMilestoneIndex(TimelockMilestoneIndexFeatureBlockDto),
    /// A timelock unix feature block.
    TimelockUnix(TimelockUnixFeatureBlockDto),
    /// An expiration milestone index feature block.
    ExpirationMilestoneIndex(ExpirationMilestoneIndexFeatureBlockDto),
    /// An expiration unix feature block.
    ExpirationUnix(ExpirationUnixFeatureBlockDto),
    /// An indexation feature block.
    Indexation(IndexationFeatureBlockDto),
    /// A metadata feature block.
    Metadata(MetadataFeatureBlockDto),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SenderFeatureBlockDto(pub AddressDto);
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssuerFeatureBlockDto(pub AddressDto);
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DustDepositReturnFeatureBlockDto(pub u64);
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelockMilestoneIndexFeatureBlockDto(pub MilestoneIndex);
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelockUnixFeatureBlockDto(pub u32);
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpirationMilestoneIndexFeatureBlockDto(pub MilestoneIndex);
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpirationUnixFeatureBlockDto(pub u32);
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexationFeatureBlockDto(pub String);
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetadataFeatureBlockDto(pub String);

impl FeatureBlockDto {
    /// Return the output kind of an `Output`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Sender(_) => SenderFeatureBlock::KIND,
            Self::Issuer(_) => IssuerFeatureBlock::KIND,
            Self::DustDepositReturn(_) => DustDepositReturnFeatureBlock::KIND,
            Self::TimelockMilestoneIndex(_) => TimelockMilestoneIndexFeatureBlock::KIND,
            Self::TimelockUnix(_) => TimelockUnixFeatureBlock::KIND,
            Self::ExpirationMilestoneIndex(_) => ExpirationMilestoneIndexFeatureBlock::KIND,
            Self::ExpirationUnix(_) => ExpirationUnixFeatureBlock::KIND,
            Self::Indexation(_) => IndexationFeatureBlock::KIND,
            Self::Metadata(_) => MetadataFeatureBlock::KIND,
        }
    }
}

impl From<&FeatureBlock> for FeatureBlockDto {
    fn from(value: &FeatureBlock) -> Self {
        match value {
            FeatureBlock::Sender(v) => Self::Sender(SenderFeatureBlockDto(v.address().into())),
            FeatureBlock::Issuer(v) => Self::Issuer(IssuerFeatureBlockDto(v.address().into())),
            FeatureBlock::DustDepositReturn(v) => Self::DustDepositReturn(DustDepositReturnFeatureBlockDto(v.amount())),
            FeatureBlock::TimelockMilestoneIndex(v) => {
                Self::TimelockMilestoneIndex(TimelockMilestoneIndexFeatureBlockDto(v.index()))
            }
            FeatureBlock::TimelockUnix(v) => Self::TimelockUnix(TimelockUnixFeatureBlockDto(v.timestamp())),
            FeatureBlock::ExpirationMilestoneIndex(v) => {
                Self::ExpirationMilestoneIndex(ExpirationMilestoneIndexFeatureBlockDto(v.index()))
            }
            FeatureBlock::ExpirationUnix(v) => Self::ExpirationUnix(ExpirationUnixFeatureBlockDto(v.timestamp())),
            FeatureBlock::Indexation(v) => Self::Indexation(IndexationFeatureBlockDto(v.to_string())),
            FeatureBlock::Metadata(v) => Self::Metadata(MetadataFeatureBlockDto(v.to_string())),
        }
    }
}

impl TryFrom<&FeatureBlockDto> for FeatureBlock {
    type Error = Error;

    fn try_from(value: &FeatureBlockDto) -> Result<Self, Self::Error> {
        Ok(match value {
            FeatureBlockDto::Sender(v) => Self::Sender(SenderFeatureBlock::new((&v.0).try_into()?)),
            FeatureBlockDto::Issuer(v) => Self::Issuer(IssuerFeatureBlock::new((&v.0).try_into()?)),
            FeatureBlockDto::DustDepositReturn(v) => Self::DustDepositReturn(DustDepositReturnFeatureBlock::new(v.0)?),
            FeatureBlockDto::TimelockMilestoneIndex(v) => {
                Self::TimelockMilestoneIndex(TimelockMilestoneIndexFeatureBlock::new(v.0))
            }
            FeatureBlockDto::TimelockUnix(v) => Self::TimelockUnix(TimelockUnixFeatureBlock::new(v.0)),
            FeatureBlockDto::ExpirationMilestoneIndex(v) => {
                Self::ExpirationMilestoneIndex(ExpirationMilestoneIndexFeatureBlock::new(v.0))
            }
            FeatureBlockDto::ExpirationUnix(v) => Self::ExpirationUnix(ExpirationUnixFeatureBlock::new(v.0)),
            FeatureBlockDto::Indexation(v) => Self::Indexation(IndexationFeatureBlock::new(
                &hex::decode(&v.0).map_err(|_e| Error::InvalidSemanticField("IndexationFeatureBlock"))?,
            )?),
            FeatureBlockDto::Metadata(v) => Self::Metadata(MetadataFeatureBlock::new(
                &hex::decode(&v.0).map_err(|_e| Error::InvalidSemanticField("MetadataFeatureBlock"))?,
            )?),
        })
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
            signatures: value.signatures().iter().map(hex::encode).collect(),
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
                        .map_err(|_| Error::InvalidSyntaxField("parentMessageIds"))?,
                );
            }
            let merkle_proof = {
                let mut buf = [0u8; MilestoneEssence::MERKLE_PROOF_LENGTH];
                hex::decode_to_slice(&value.inclusion_merkle_proof, &mut buf)
                    .map_err(|_| Error::InvalidSyntaxField("inclusionMerkleProof"))?;
                buf
            };
            let next_pow_score = value.next_pow_score;
            let next_pow_score_milestone_index = value.next_pow_score_milestone_index;
            let mut public_keys = Vec::new();
            for v in &value.public_keys {
                let key = {
                    let mut buf = [0u8; MilestoneEssence::PUBLIC_KEY_LENGTH];
                    hex::decode_to_slice(v, &mut buf).map_err(|_| Error::InvalidSyntaxField("publicKeys"))?;
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
                    .map_err(|_| Error::InvalidSyntaxField("signatures"))?
                    .try_into()
                    .map_err(|_| Error::InvalidSyntaxField("signatures"))?,
            )
        }

        Ok(MilestonePayload::new(essence, signatures)?)
    }
}

/// The payload type to define a indexation payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexationPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: String,
    pub data: String,
}

impl From<&IndexationPayload> for IndexationPayloadDto {
    fn from(value: &IndexationPayload) -> Self {
        IndexationPayloadDto {
            kind: IndexationPayload::KIND,
            index: hex::encode(value.index()),
            data: hex::encode(value.data()),
        }
    }
}

impl TryFrom<&IndexationPayloadDto> for IndexationPayload {
    type Error = Error;

    fn try_from(value: &IndexationPayloadDto) -> Result<Self, Self::Error> {
        Ok(IndexationPayload::new(
            &hex::decode(value.index.clone()).map_err(|_| Error::InvalidSyntaxField("index"))?,
            &hex::decode(value.data.clone()).map_err(|_| Error::InvalidSyntaxField("data"))?,
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
            funds: value.funds().iter().map(|m| m.into()).collect::<_>(),
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
            value.funds.iter().map(|m| m.try_into()).collect::<Result<_, _>>()?,
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
            address: value.output().address().into(),
            deposit: value.output().amount(),
        }
    }
}

impl TryFrom<&MigratedFundsEntryDto> for MigratedFundsEntry {
    type Error = Error;

    fn try_from(value: &MigratedFundsEntryDto) -> Result<Self, Self::Error> {
        let mut tail_transaction_hash = [0u8; TailTransactionHash::LENGTH];
        hex::decode_to_slice(&value.tail_transaction_hash, &mut tail_transaction_hash)
            .map_err(|_| Error::InvalidSyntaxField("tailTransactionHash"))?;
        Ok(MigratedFundsEntry::new(
            TailTransactionHash::new(tail_transaction_hash)?,
            SimpleOutput::new((&value.address).try_into()?, value.deposit)?,
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
