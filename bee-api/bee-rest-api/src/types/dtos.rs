// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::error::Error;

use bee_message::{
    address::{Address, BlsAddress, Ed25519Address},
    input::{Input, UtxoInput},
    output::{AssetBalance, AssetId, Output, OutputId, SignatureLockedAssetOutput, SignatureLockedSingleOutput},
    parents::{Parent, Parents},
    payload::{
        data::DataPayload,
        drng::{
            ApplicationMessagePayload, BeaconPayload, CollectiveBeaconPayload, DkgPayload, EncryptedDeal,
            BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH, BEACON_PARTIAL_PUBLIC_KEY_LENGTH, BEACON_SIGNATURE_LENGTH,
        },
        fpc::{Conflict, FpcPayload, Opinion, Timestamp},
        indexation::IndexationPayload,
        salt_declaration::{Salt, SaltDeclarationPayload},
        transaction::{TransactionEssence, TransactionId, TransactionPayload, PLEDGE_ID_LENGTH},
        Payload,
    },
    signature::{BlsSignature, Ed25519Signature, Signature},
    unlock::{ReferenceUnlock, SignatureUnlock, UnlockBlock, UnlockBlocks},
    Message, MessageBuilder, MessageId, MESSAGE_PUBLIC_KEY_LENGTH, MESSAGE_SIGNATURE_LENGTH,
};

use bee_message::payload::MessagePayload;
use bee_packable::PackableExt;

use serde::{de::Error as DeError, Deserialize, Serialize, Serializer};
use serde_json::Value;

use std::{
    convert::{TryFrom, TryInto},
    ops::Deref,
    str::FromStr,
};

#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageDto {
    #[serde(rename = "parentMessageIds")]
    pub parents: Vec<ParentDto>,
    #[serde(rename = "issuerPublicKey")]
    pub issuer_public_key: String, //[u8; MESSAGE_PUBLIC_KEY_LENGTH],
    #[serde(rename = "issueTimestamp")]
    pub issue_timestamp: String,
    #[serde(rename = "sequenceNumber")]
    pub sequence_number: String,
    pub payload: Option<PayloadDto>,
    pub nonce: String,
    pub signature: String, //[u8; MESSAGE_SIGNATURE_LENGTH],
}

impl From<&Message> for MessageDto {
    fn from(value: &Message) -> Self {
        MessageDto {
            parents: value.parents().iter().map(Into::into).collect(),
            issuer_public_key: hex::encode(value.issuer_public_key()),
            issue_timestamp: value.issue_timestamp().to_string(),
            sequence_number: value.sequence_number().to_string(),
            payload: value.payload().map(PayloadDto::from),
            nonce: value.nonce().to_string(),
            signature: hex::encode(value.signature()),
        }
    }
}

impl TryFrom<&MessageDto> for Message {
    type Error = Error;

    fn try_from(value: &MessageDto) -> Result<Self, Self::Error> {
        let mut builder = MessageBuilder::new()
            .with_parents(Parents::new(
                value
                    .parents
                    .iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<Parent>, Error>>()?,
            )?)
            .with_issuer_public_key({
                let mut public_key: [u8; MESSAGE_PUBLIC_KEY_LENGTH] = [0u8; MESSAGE_PUBLIC_KEY_LENGTH];

                hex::decode_to_slice(&value.issuer_public_key, &mut public_key)
                    .map_err(|_| Error::InvalidSyntaxField("issuerPublicKey"))?;

                public_key
            })
            .with_issue_timestamp(
                value
                    .issue_timestamp
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidSyntaxField("issueTimestamp"))?,
            )
            .with_sequence_number(
                value
                    .sequence_number
                    .parse::<u32>()
                    .map_err(|_| Error::InvalidSyntaxField("sequenceNumber"))?,
            )
            .with_nonce(
                value
                    .nonce
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidSyntaxField("nonce"))?,
            )
            .with_signature({
                let mut signature: [u8; MESSAGE_SIGNATURE_LENGTH] = [0; MESSAGE_SIGNATURE_LENGTH];

                hex::decode_to_slice(&value.signature, &mut signature)
                    .map_err(|_| Error::InvalidSyntaxField("signature"))?;

                signature
            });
        if let Some(p) = value.payload.as_ref() {
            builder = builder.with_payload(p.try_into()?);
        }

        Ok(builder.finish()?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParentDto {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "messageId")]
    pub message_id: String,
}

impl From<&Parent> for ParentDto {
    fn from(value: &Parent) -> Self {
        match value {
            Parent::Strong(m) => ParentDto {
                kind: 0,
                message_id: hex::encode(m),
            },
            Parent::Weak(m) => ParentDto {
                kind: 1,
                message_id: hex::encode(m),
            },
        }
    }
}

impl TryFrom<&ParentDto> for Parent {
    type Error = Error;

    fn try_from(value: &ParentDto) -> Result<Self, Self::Error> {
        let message_id = value
            .message_id
            .parse::<MessageId>()
            .map_err(|_| Error::InvalidSyntaxField("messageId"))?;
        match value.kind {
            0 => Ok(Parent::Strong(message_id)),
            1 => Ok(Parent::Weak(message_id)),
            _ => Err(Error::InvalidSyntaxField("kind")),
        }
    }
}

/// Describes all the different payload types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PayloadDto {
    Data(Box<DataPayloadDto>),
    Transaction(Box<TransactionPayloadDto>),
    Fpc(Box<FpcPayloadDto>),
    ApplicationMessage(Box<ApplicationMessagePayloadDto>),
    Dkg(Box<DkgPayloadDto>),
    Beacon(Box<BeaconPayloadDto>),
    CollectiveBeacon(Box<CollectiveBeaconPayloadDto>),
    SaltDeclaration(Box<SaltDeclarationPayloadDto>),
    Indexation(Box<IndexationPayloadDto>),
}

impl From<&Payload> for PayloadDto {
    fn from(value: &Payload) -> Self {
        match value {
            Payload::Data(d) => PayloadDto::Data(Box::new(DataPayloadDto::from(d.as_ref()))),
            Payload::Transaction(t) => PayloadDto::Transaction(Box::new(TransactionPayloadDto::from(t.as_ref()))),
            Payload::Fpc(f) => PayloadDto::Fpc(Box::new(FpcPayloadDto::from(f.as_ref()))),
            Payload::ApplicationMessage(a) => {
                PayloadDto::ApplicationMessage(Box::new(ApplicationMessagePayloadDto::from(a.as_ref())))
            }
            Payload::Dkg(d) => PayloadDto::Dkg(Box::new(DkgPayloadDto::from(d.as_ref()))),
            Payload::Beacon(b) => PayloadDto::Beacon(Box::new(BeaconPayloadDto::from(b.as_ref()))),
            Payload::CollectiveBeacon(c) => {
                PayloadDto::CollectiveBeacon(Box::new(CollectiveBeaconPayloadDto::from(c.as_ref())))
            }
            Payload::SaltDeclaration(s) => {
                PayloadDto::SaltDeclaration(Box::new(SaltDeclarationPayloadDto::from(s.as_ref())))
            }
            Payload::Indexation(i) => PayloadDto::Indexation(Box::new(IndexationPayloadDto::from(i.as_ref()))),
        }
    }
}

impl TryFrom<&PayloadDto> for Payload {
    type Error = Error;
    fn try_from(value: &PayloadDto) -> Result<Self, Self::Error> {
        Ok(match value {
            PayloadDto::Data(d) => Payload::Data(Box::new(DataPayload::try_from(d.as_ref())?)),
            PayloadDto::Transaction(t) => Payload::Transaction(Box::new(TransactionPayload::try_from(t.as_ref())?)),
            PayloadDto::Fpc(f) => Payload::Fpc(Box::new(FpcPayload::try_from(f.as_ref())?)),
            PayloadDto::ApplicationMessage(a) => {
                Payload::ApplicationMessage(Box::new(ApplicationMessagePayload::try_from(a.as_ref())?))
            }
            PayloadDto::Dkg(d) => Payload::Dkg(Box::new(DkgPayload::try_from(d.as_ref())?)),
            PayloadDto::Beacon(b) => Payload::Beacon(Box::new(BeaconPayload::try_from(b.as_ref())?)),
            PayloadDto::CollectiveBeacon(c) => {
                Payload::CollectiveBeacon(Box::new(CollectiveBeaconPayload::try_from(c.as_ref())?))
            }
            PayloadDto::SaltDeclaration(s) => {
                Payload::SaltDeclaration(Box::new(SaltDeclarationPayload::try_from(s.as_ref())?))
            }
            PayloadDto::Indexation(i) => Payload::Indexation(Box::new(IndexationPayload::try_from(i.as_ref())?)),
        })
    }
}

/// The payload type to define a value transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub transaction_essence: TransactionEssenceDto,
    #[serde(rename = "unlockBlocks")]
    pub unlock_blocks: Vec<UnlockBlockDto>,
}

impl From<&TransactionPayload> for TransactionPayloadDto {
    fn from(value: &TransactionPayload) -> Self {
        TransactionPayloadDto {
            kind: TransactionPayload::KIND,
            transaction_essence: value.essence().into(),
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
            .with_essence((&value.transaction_essence).try_into()?)
            .with_unlock_blocks(UnlockBlocks::new(unlock_blocks)?);

        Ok(builder.finish()?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionEssenceDto {
    pub timestamp: String,
    pub access_pledge_id: String,    //[u8; PLEDGE_ID_LENGTH],
    pub consensus_pledge_id: String, //[u8; PLEDGE_ID_LENGTH],
    pub inputs: Vec<InputDto>,
    pub outputs: Vec<OutputDto>,
    pub payload: Option<PayloadDto>,
}

impl From<&TransactionEssence> for TransactionEssenceDto {
    fn from(value: &TransactionEssence) -> Self {
        TransactionEssenceDto {
            timestamp: value.timestamp().to_string(),
            access_pledge_id: hex::encode(value.access_pledge_id()),
            consensus_pledge_id: hex::encode(value.consensus_pledge_id()),
            inputs: value.inputs().iter().map(Into::into).collect::<Vec<_>>(),
            outputs: value.outputs().iter().map(Into::into).collect::<Vec<_>>(),
            payload: value.payload().as_ref().map(PayloadDto::from),
        }
    }
}

impl TryFrom<&TransactionEssenceDto> for TransactionEssence {
    type Error = Error;

    fn try_from(value: &TransactionEssenceDto) -> Result<Self, Self::Error> {
        let mut builder = TransactionEssence::builder()
            .with_timestamp(
                value
                    .timestamp
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidSyntaxField("issueTimestamp"))?,
            )
            .with_access_pledge_id({
                let mut access_pledge_id: [u8; PLEDGE_ID_LENGTH] = [0; PLEDGE_ID_LENGTH];

                hex::decode_to_slice(&value.access_pledge_id, &mut access_pledge_id)
                    .map_err(|_| Error::InvalidSyntaxField("accessPledgeId"))?;

                access_pledge_id
            })
            .with_consensus_pledge_id({
                let mut consensus_pledge_id: [u8; PLEDGE_ID_LENGTH] = [0; PLEDGE_ID_LENGTH];

                hex::decode_to_slice(&value.consensus_pledge_id, &mut consensus_pledge_id)
                    .map_err(|_| Error::InvalidSyntaxField("consensusPledgeId"))?;

                consensus_pledge_id
            });

        for i in &value.inputs {
            builder = builder.add_input(i.try_into()?);
        }

        for o in &value.outputs {
            builder = builder.add_output(o.try_into()?);
        }

        if let Some(p) = &value.payload {
            builder = builder.with_payload(Payload::try_from(p).map_err(|_| Error::InvalidSemanticField("payload"))?);
        }

        Ok(builder.finish()?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UnlockBlockDto {
    Signature(SignatureUnlockDto),
    Reference(ReferenceUnlockDto),
}

impl From<&UnlockBlock> for UnlockBlockDto {
    fn from(value: &UnlockBlock) -> Self {
        match value {
            UnlockBlock::Signature(s) => UnlockBlockDto::Signature(s.into()),
            UnlockBlock::Reference(r) => UnlockBlockDto::Reference(r.into()),
        }
    }
}

impl TryFrom<&UnlockBlockDto> for UnlockBlock {
    type Error = Error;

    fn try_from(value: &UnlockBlockDto) -> Result<Self, Self::Error> {
        match value {
            UnlockBlockDto::Signature(s) => Ok(UnlockBlock::Signature(s.try_into()?)),
            UnlockBlockDto::Reference(r) => Ok(UnlockBlock::Reference(r.try_into()?)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureUnlockDto(SignatureDto);

impl From<&SignatureUnlock> for SignatureUnlockDto {
    fn from(value: &SignatureUnlock) -> Self {
        SignatureUnlockDto(value.signature().into())
    }
}

impl TryFrom<&SignatureUnlockDto> for SignatureUnlock {
    type Error = Error;

    fn try_from(value: &SignatureUnlockDto) -> Result<Self, Self::Error> {
        Ok(SignatureUnlock::new(Signature::try_from(&value.0)?))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferenceUnlockDto(u16);

impl From<&ReferenceUnlock> for ReferenceUnlockDto {
    fn from(value: &ReferenceUnlock) -> Self {
        ReferenceUnlockDto(value.index())
    }
}

impl TryFrom<&ReferenceUnlockDto> for ReferenceUnlock {
    type Error = Error;

    fn try_from(value: &ReferenceUnlockDto) -> Result<Self, Self::Error> {
        value.0.try_into().map_err(|_| Error::InvalidSemanticField("index"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SignatureDto {
    Ed25519(Ed25519SignatureDto),
    Bls(BlsSignatureDto),
}

impl From<&Signature> for SignatureDto {
    fn from(value: &Signature) -> Self {
        match value {
            Signature::Ed25519(e) => SignatureDto::Ed25519(e.into()),
            Signature::Bls(b) => SignatureDto::Bls(b.into()),
        }
    }
}

impl TryFrom<&SignatureDto> for Signature {
    type Error = Error;

    fn try_from(value: &SignatureDto) -> Result<Self, Self::Error> {
        match value {
            SignatureDto::Ed25519(e) => Ok(Signature::Ed25519(e.try_into()?)),
            SignatureDto::Bls(b) => Ok(Signature::Bls(b.try_into()?)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ed25519SignatureDto {
    #[serde(rename = "type")]
    pub kind: u8,
    public_key: String, //[u8; Self::PUBLIC_KEY_LENGTH],
    signature: String,  //[u8; Self::SIGNATURE_LENGTH],
}

impl From<&Ed25519Signature> for Ed25519SignatureDto {
    fn from(value: &Ed25519Signature) -> Self {
        Ed25519SignatureDto {
            kind: Ed25519Signature::KIND,
            public_key: hex::encode(value.public_key()),
            signature: hex::encode(value.signature()),
        }
    }
}

impl TryFrom<&Ed25519SignatureDto> for Ed25519Signature {
    type Error = Error;

    fn try_from(value: &Ed25519SignatureDto) -> Result<Self, Self::Error> {
        Ok(Ed25519Signature::new(
            {
                let mut public_key: [u8; Ed25519Signature::PUBLIC_KEY_LENGTH] =
                    [0; Ed25519Signature::PUBLIC_KEY_LENGTH];

                hex::decode_to_slice(&value.public_key, &mut public_key)
                    .map_err(|_| Error::InvalidSyntaxField("publicKey"))?;

                public_key
            },
            {
                let mut signature: [u8; Ed25519Signature::SIGNATURE_LENGTH] = [0; Ed25519Signature::SIGNATURE_LENGTH];

                hex::decode_to_slice(&value.signature, &mut signature)
                    .map_err(|_| Error::InvalidSyntaxField("publicKey"))?;

                signature
            },
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlsSignatureDto(String); //[u8; Self::LENGTH]

impl From<&BlsSignature> for BlsSignatureDto {
    fn from(value: &BlsSignature) -> Self {
        BlsSignatureDto(hex::encode(value.as_ref()))
    }
}

impl TryFrom<&BlsSignatureDto> for BlsSignature {
    type Error = Error;

    fn try_from(value: &BlsSignatureDto) -> Result<Self, Self::Error> {
        Ok(BlsSignature::new({
            let mut bytes: [u8; BlsSignature::LENGTH] = [0; BlsSignature::LENGTH];

            hex::decode_to_slice(&value.0, &mut bytes).map_err(|_| Error::InvalidSyntaxField("bytes"))?;

            bytes
        }))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InputDto {
    Utxo(UtxoInputDto),
}

impl From<&Input> for InputDto {
    fn from(value: &Input) -> Self {
        match value {
            Input::Utxo(u) => InputDto::Utxo(UtxoInputDto {
                kind: UtxoInput::KIND,
                transaction_id: u.output_id().transaction_id().to_string(),
                transaction_output_index: u.output_id().index(),
            }),
        }
    }
}

impl TryFrom<&InputDto> for Input {
    type Error = Error;

    fn try_from(value: &InputDto) -> Result<Self, Self::Error> {
        match value {
            InputDto::Utxo(i) => Ok(Input::Utxo(UtxoInput::new(OutputId::new(
                i.transaction_id
                    .parse::<TransactionId>()
                    .map_err(|_| Error::InvalidSyntaxField("transactionId"))?,
                i.transaction_output_index,
            )?))),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtxoInputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "transactionOutputIndex")]
    pub transaction_output_index: u16,
}

#[derive(Clone, Debug)]
pub enum OutputDto {
    SignatureLockedSingle(SignatureLockedSingleOutputDto),
    SignatureLockedAsset(SignatureLockedAssetOutputDto),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureLockedSingleOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    address: AddressDto,
    amount: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureLockedAssetOutputDto {
    #[serde(rename = "type")]
    pub kind: u8,
    address: AddressDto,
    balances: Vec<AssetBalanceDto>,
}

impl From<&Output> for OutputDto {
    fn from(value: &Output) -> Self {
        match value {
            Output::SignatureLockedSingle(s) => OutputDto::SignatureLockedSingle(SignatureLockedSingleOutputDto {
                kind: SignatureLockedSingleOutput::KIND,
                address: s.address().into(),
                amount: s.amount(),
            }),
            Output::SignatureLockedAsset(s) => OutputDto::SignatureLockedAsset(SignatureLockedAssetOutputDto {
                kind: SignatureLockedAssetOutput::KIND,
                address: s.address().into(),
                balances: s
                    .balance_iter()
                    .map(|b| AssetBalanceDto {
                        id: hex::encode(b.id().deref()),
                        balance: b.balance(),
                    })
                    .collect(),
            }),
        }
    }
}

impl TryFrom<&OutputDto> for Output {
    type Error = Error;

    fn try_from(value: &OutputDto) -> Result<Self, Self::Error> {
        match value {
            OutputDto::SignatureLockedSingle(s) => Ok(Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(
                (&s.address).try_into()?,
                s.amount,
            )?)),
            OutputDto::SignatureLockedAsset(s) => Ok(Output::SignatureLockedAsset(SignatureLockedAssetOutput::new(
                (&s.address).try_into()?,
                s.balances
                    .iter()
                    .map(|b| {
                        let mut asset_id: [u8; AssetId::LENGTH] = [0; AssetId::LENGTH];

                        match hex::decode_to_slice(&b.id, &mut asset_id)
                            .map_err(|_| Error::InvalidSyntaxField("assetId"))
                        {
                            Ok(()) => (),
                            Err(e) => return Err(e),
                        }

                        Ok(AssetBalance::new(asset_id.into(), b.balance))
                    })
                    .collect::<Result<Vec<AssetBalance>, _>>()?,
            )?)),
        }
    }
}

impl<'de> serde::Deserialize<'de> for OutputDto {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        Ok(match value.get("type").and_then(Value::as_u64).unwrap() as u8 {
            SignatureLockedSingleOutput::KIND => {
                OutputDto::SignatureLockedSingle(SignatureLockedSingleOutputDto::deserialize(value).unwrap())
            }
            SignatureLockedAssetOutput::KIND => {
                OutputDto::SignatureLockedAsset(SignatureLockedAssetOutputDto::deserialize(value).unwrap())
            }
            type_ => return Err(DeError::custom(format!("unsupported type {:?}", type_))),
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
            T2(&'a SignatureLockedAssetOutputDto),
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
            OutputDto::SignatureLockedAsset(s) => TypedOutput {
                output: OutputDto_::T2(s),
            },
        };
        output.serialize(serializer)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddressDto {
    Ed25519(Ed25519AddressDto),
    Bls(BlsAddressDto),
}

impl From<&Address> for AddressDto {
    fn from(value: &Address) -> Self {
        match value {
            Address::Ed25519(ed) => AddressDto::Ed25519(ed.into()),
            Address::Bls(b) => AddressDto::Bls(BlsAddressDto(hex::encode(b.as_ref()))),
        }
    }
}

impl TryFrom<&AddressDto> for Address {
    type Error = Error;

    fn try_from(value: &AddressDto) -> Result<Self, Self::Error> {
        match value {
            AddressDto::Ed25519(a) => Ok(Address::Ed25519(a.try_into()?)),
            AddressDto::Bls(b) => Ok(Address::Bls(BlsAddress::from_str(&b.0)?)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ed25519AddressDto {
    #[serde(rename = "type")]
    pub kind: u8,
    pub address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlsAddressDto(String);

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetBalanceDto {
    id: String, //[u8; Self::LENGTH],
    balance: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    data: String,
}

impl From<&DataPayload> for DataPayloadDto {
    fn from(value: &DataPayload) -> Self {
        DataPayloadDto {
            kind: DataPayload::KIND,
            data: hex::encode(value.data()),
        }
    }
}

impl TryFrom<&DataPayloadDto> for DataPayload {
    type Error = Error;

    fn try_from(value: &DataPayloadDto) -> Result<Self, Self::Error> {
        if value.data.len() % 2 != 0 {
            return Err(Error::InvalidSyntaxField("data"));
        }
        match (0..value.data.len())
            .step_by(2)
            .map(|i| {
                value
                    .data
                    .get(i..i + 2)
                    .and_then(|sub| u8::from_str_radix(sub, 16).ok())
            })
            .collect()
        {
            Some(data) => Ok(DataPayload::new(data).map_err(|_| Error::InvalidSyntaxField("data"))?),
            None => Err(Error::InvalidSyntaxField("data")),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FpcPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    conflicts: Vec<ConflictDto>,
    timestamps: Vec<TimestampDto>,
}

impl From<&FpcPayload> for FpcPayloadDto {
    fn from(value: &FpcPayload) -> Self {
        FpcPayloadDto {
            kind: FpcPayload::KIND,
            conflicts: value.conflicts().map(Into::into).collect(),
            timestamps: value.timestamps().map(Into::into).collect(),
        }
    }
}

impl TryFrom<&FpcPayloadDto> for FpcPayload {
    type Error = Error;

    fn try_from(value: &FpcPayloadDto) -> Result<Self, Self::Error> {
        let builder = FpcPayload::builder()
            .with_conflicts(
                value
                    .conflicts
                    .iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<Conflict>, Error>>()?,
            )
            .with_timestamps(
                value
                    .timestamps
                    .iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<Timestamp>, Error>>()?,
            );

        Ok(builder.finish()?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConflictDto {
    /// Identifier of the conflicting transaction.
    transaction_id: String,
    /// The node's opinion value in a given round.
    opinion: u8,
    /// Voting round number.
    round: u8,
}

impl From<&Conflict> for ConflictDto {
    fn from(value: &Conflict) -> Self {
        ConflictDto {
            transaction_id: hex::encode(value.transaction_id()),
            opinion: value.opinion() as u8,
            round: value.round(),
        }
    }
}

impl TryFrom<&ConflictDto> for Conflict {
    type Error = Error;

    fn try_from(value: &ConflictDto) -> Result<Self, Self::Error> {
        Ok(Conflict::new(
            TransactionId::from_str(&value.transaction_id).map_err(|_| Error::InvalidSyntaxField("transactionId"))?,
            match Opinion::unpack_verified(&[value.opinion]) {
                Ok(Opinion::Like) => Ok(Opinion::Like),
                Ok(Opinion::Dislike) => Ok(Opinion::Dislike),
                Ok(Opinion::Unknown) => Ok(Opinion::Unknown),
                _ => Err(Error::InvalidSyntaxField("opinion")),
            }?,
            value.round,
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimestampDto {
    /// Identifier of the message that contains the timestamp.
    message_id: String,
    /// The node's opinion value in a given round.
    opinion: u8,
    /// Voting round number.
    round: u8,
}

impl From<&Timestamp> for TimestampDto {
    fn from(value: &Timestamp) -> Self {
        TimestampDto {
            message_id: hex::encode(value.message_id().as_ref()),
            opinion: value.opinion() as u8,
            round: value.round(),
        }
    }
}

impl TryFrom<&TimestampDto> for Timestamp {
    type Error = Error;

    fn try_from(value: &TimestampDto) -> Result<Self, Self::Error> {
        Ok(Timestamp::new(
            MessageId::from_str(&value.message_id).map_err(|_| Error::InvalidSyntaxField("messageId"))?,
            match Opinion::unpack_verified(&[value.opinion]) {
                Ok(Opinion::Like) => Ok(Opinion::Like),
                Ok(Opinion::Dislike) => Ok(Opinion::Dislike),
                Ok(Opinion::Unknown) => Ok(Opinion::Unknown),
                _ => Err(Error::InvalidSyntaxField("opinion")),
            }?,
            value.round,
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationMessagePayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    instance_id: String, // u32,
}

impl From<&ApplicationMessagePayload> for ApplicationMessagePayloadDto {
    fn from(value: &ApplicationMessagePayload) -> Self {
        ApplicationMessagePayloadDto {
            kind: ApplicationMessagePayload::KIND,
            instance_id: value.instance_id().to_string(),
        }
    }
}

impl TryFrom<&ApplicationMessagePayloadDto> for ApplicationMessagePayload {
    type Error = Error;

    fn try_from(value: &ApplicationMessagePayloadDto) -> Result<Self, Self::Error> {
        Ok(ApplicationMessagePayload::new(
            value
                .instance_id
                .parse::<u32>()
                .map_err(|_| Error::InvalidSyntaxField("instanceId"))?,
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DkgPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    instance_id: String,
    from_index: String,
    to_index: String,
    deal: EncryptedDealDto,
}

impl From<&DkgPayload> for DkgPayloadDto {
    fn from(value: &DkgPayload) -> Self {
        DkgPayloadDto {
            kind: DkgPayload::KIND,
            instance_id: value.instance_id().to_string(),
            from_index: value.from_index().to_string(),
            to_index: value.to_index().to_string(),
            deal: value.deal().into(),
        }
    }
}

impl TryFrom<&DkgPayloadDto> for DkgPayload {
    type Error = Error;

    fn try_from(value: &DkgPayloadDto) -> Result<Self, Self::Error> {
        let builder = DkgPayload::builder()
            .with_instance_id(
                value
                    .instance_id
                    .parse::<u32>()
                    .map_err(|_| Error::InvalidSyntaxField("instanceId"))?,
            )
            .with_from_index(
                value
                    .from_index
                    .parse::<u32>()
                    .map_err(|_| Error::InvalidSyntaxField("fromIndex"))?,
            )
            .with_to_index(
                value
                    .to_index
                    .parse::<u32>()
                    .map_err(|_| Error::InvalidSyntaxField("toIndex"))?,
            )
            .with_deal(EncryptedDeal::try_from(&value.deal)?);

        Ok(builder.finish()?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedDealDto {
    dh_key: String,          // VecPrefix<u8, BoundedU32<0, PREFIXED_LENGTH_MAX>>,
    nonce: String,           // VecPrefix<u8, BoundedU32<0, PREFIXED_LENGTH_MAX>>,
    encrypted_share: String, // VecPrefix<u8, BoundedU32<0, PREFIXED_LENGTH_MAX>>,
    threshold: String,       // u32,
    commitments: String,     // VecPrefix<u8, BoundedU32<0, PREFIXED_LENGTH_MAX>>,
}

impl From<&EncryptedDeal> for EncryptedDealDto {
    fn from(value: &EncryptedDeal) -> Self {
        EncryptedDealDto {
            dh_key: hex::encode(value.dh_key()),
            nonce: hex::encode(value.nonce()),
            encrypted_share: hex::encode(value.encrypted_share()),
            threshold: value.threshold().to_string(),
            commitments: hex::encode(value.commitments()),
        }
    }
}

impl TryFrom<&EncryptedDealDto> for EncryptedDeal {
    type Error = Error;

    fn try_from(value: &EncryptedDealDto) -> Result<Self, Self::Error> {
        let builder = EncryptedDeal::builder()
            .with_dh_key(match (0..value.dh_key.len())
                .step_by(2)
                .map(|i| {
                    value
                        .dh_key
                        .get(i..i + 2)
                        .and_then(|sub| u8::from_str_radix(sub, 16).ok())
                })
                .collect()
            {
                Some(dh_key) => Ok(dh_key),
                None => Err(Error::InvalidSyntaxField("dhKey")),
            }?)
            .with_nonce(match (0..value.nonce.len())
                .step_by(2)
                .map(|i| {
                    value
                        .nonce
                        .get(i..i + 2)
                        .and_then(|sub| u8::from_str_radix(sub, 16).ok())
                })
                .collect()
            {
                Some(nonce) => Ok(nonce),
                None => Err(Error::InvalidSyntaxField("nonce")),
            }?)
            .with_encrypted_share(match (0..value.encrypted_share.len())
                .step_by(2)
                .map(|i| {
                    value
                        .encrypted_share
                        .get(i..i + 2)
                        .and_then(|sub| u8::from_str_radix(sub, 16).ok())
                })
                .collect()
            {
                Some(encrypted_share) => Ok(encrypted_share),
                None => Err(Error::InvalidSyntaxField("encryptedShare")),
            }?)
            .with_threshold(
                value
                    .threshold
                    .parse::<u32>()
                    .map_err(|_| Error::InvalidSyntaxField("threshold"))?,
            )
            .with_commitments(match (0..value.commitments.len())
                .step_by(2)
                .map(|i| {
                    value
                        .commitments
                        .get(i..i + 2)
                        .and_then(|sub| u8::from_str_radix(sub, 16).ok())
                })
                .collect()
            {
                Some(commitments) => Ok(commitments),
                None => Err(Error::InvalidSyntaxField("commitments")),
            }?);

        Ok(builder.finish()?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeaconPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    instance_id: String,        // u32,
    round: String,              // u64,
    partial_public_key: String, //[u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH],
    partial_signature: String,  //[u8; BEACON_SIGNATURE_LENGTH],
}

impl From<&BeaconPayload> for BeaconPayloadDto {
    fn from(value: &BeaconPayload) -> Self {
        BeaconPayloadDto {
            kind: BeaconPayload::KIND,
            instance_id: value.instance_id().to_string(),
            round: value.round().to_string(),
            partial_public_key: hex::encode(value.partial_public_key()),
            partial_signature: hex::encode(value.partial_signature()),
        }
    }
}

impl TryFrom<&BeaconPayloadDto> for BeaconPayload {
    type Error = Error;

    fn try_from(value: &BeaconPayloadDto) -> Result<Self, Self::Error> {
        let builder = BeaconPayload::builder()
            .with_instance_id(
                value
                    .instance_id
                    .parse::<u32>()
                    .map_err(|_| Error::InvalidSyntaxField("instanceId"))?,
            )
            .with_round(
                value
                    .round
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidSyntaxField("round"))?,
            )
            .with_partial_public_key({
                let mut partial_public_key: [u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH] =
                    [0; BEACON_PARTIAL_PUBLIC_KEY_LENGTH];

                hex::decode_to_slice(&value.partial_public_key, &mut partial_public_key)
                    .map_err(|_| Error::InvalidSyntaxField("partialPublicKey"))?;

                partial_public_key
            })
            .with_partial_signature({
                let mut partial_signature: [u8; BEACON_SIGNATURE_LENGTH] = [0; BEACON_SIGNATURE_LENGTH];

                hex::decode_to_slice(&value.partial_signature, &mut partial_signature)
                    .map_err(|_| Error::InvalidSyntaxField("partialSignature"))?;

                partial_signature
            });

        Ok(builder.finish()?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CollectiveBeaconPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    instance_id: String,            // u32,
    round: String,                  // u64,
    prev_signature: String,         //[u8; BEACON_SIGNATURE_LENGTH],
    signature: String,              //[u8; BEACON_SIGNATURE_LENGTH],
    distributed_public_key: String, //[u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH],
}

impl From<&CollectiveBeaconPayload> for CollectiveBeaconPayloadDto {
    fn from(value: &CollectiveBeaconPayload) -> Self {
        CollectiveBeaconPayloadDto {
            kind: CollectiveBeaconPayload::KIND,
            instance_id: value.instance_id().to_string(),
            round: value.round().to_string(),
            prev_signature: hex::encode(value.prev_signature()),
            signature: hex::encode(value.signature()),
            distributed_public_key: hex::encode(value.distributed_public_key()),
        }
    }
}

impl TryFrom<&CollectiveBeaconPayloadDto> for CollectiveBeaconPayload {
    type Error = Error;

    fn try_from(value: &CollectiveBeaconPayloadDto) -> Result<Self, Self::Error> {
        let builder = CollectiveBeaconPayload::builder()
            .with_instance_id(
                value
                    .instance_id
                    .parse::<u32>()
                    .map_err(|_| Error::InvalidSyntaxField("instanceId"))?,
            )
            .with_round(
                value
                    .round
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidSyntaxField("round"))?,
            )
            .with_prev_signature({
                let mut prev_signature: [u8; BEACON_SIGNATURE_LENGTH] = [0; BEACON_SIGNATURE_LENGTH];

                hex::decode_to_slice(&value.prev_signature, &mut prev_signature)
                    .map_err(|_| Error::InvalidSyntaxField("prevSignature"))?;

                prev_signature
            })
            .with_signature({
                let mut signature: [u8; BEACON_SIGNATURE_LENGTH] = [0; BEACON_SIGNATURE_LENGTH];

                hex::decode_to_slice(&value.signature, &mut signature)
                    .map_err(|_| Error::InvalidSyntaxField("signature"))?;

                signature
            })
            .with_distributed_public_key({
                let mut distributed_public_key: [u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH] =
                    [0; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH];

                hex::decode_to_slice(&value.distributed_public_key, &mut distributed_public_key)
                    .map_err(|_| Error::InvalidSyntaxField("distributedPublicKey"))?;

                distributed_public_key
            });

        Ok(builder.finish()?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaltDeclarationPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    node_id: String, // u32,
    salt: SaltDto,
    timestamp: String, // u64,
    signature: String, //[u8; Ed25519Signature::SIGNATURE_LENGTH],
}

impl From<&SaltDeclarationPayload> for SaltDeclarationPayloadDto {
    fn from(value: &SaltDeclarationPayload) -> Self {
        SaltDeclarationPayloadDto {
            kind: SaltDeclarationPayload::KIND,
            node_id: value.node_id().to_string(),
            salt: value.salt().into(),
            timestamp: value.timestamp().to_string(),
            signature: hex::encode(value.signature()),
        }
    }
}

impl TryFrom<&SaltDeclarationPayloadDto> for SaltDeclarationPayload {
    type Error = Error;

    fn try_from(value: &SaltDeclarationPayloadDto) -> Result<Self, Self::Error> {
        let builder = SaltDeclarationPayload::builder()
            .with_node_id(
                value
                    .node_id
                    .parse::<u32>()
                    .map_err(|_| Error::InvalidSyntaxField("nodeId"))?,
            )
            .with_salt(Salt::try_from(&value.salt)?)
            .with_timestamp(
                value
                    .timestamp
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidSyntaxField("timestamp"))?,
            )
            .with_signature({
                let mut signature: [u8; Ed25519Signature::SIGNATURE_LENGTH] = [0; Ed25519Signature::SIGNATURE_LENGTH];

                hex::decode_to_slice(&value.signature, &mut signature)
                    .map_err(|_| Error::InvalidSyntaxField("signature"))?;

                signature
            });

        Ok(builder.finish()?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaltDto {
    bytes: String,       // VecPrefix<u8, BoundedU32<0, PREFIXED_BYTES_LENGTH_MAX>>,
    expiry_time: String, // u64,
}

impl From<&Salt> for SaltDto {
    fn from(value: &Salt) -> Self {
        SaltDto {
            bytes: hex::encode(value.bytes()),
            expiry_time: value.expiry_time().to_string(),
        }
    }
}

impl TryFrom<&SaltDto> for Salt {
    type Error = Error;

    fn try_from(value: &SaltDto) -> Result<Self, Self::Error> {
        Ok(Salt::new(
            match (0..value.bytes.len())
                .step_by(2)
                .map(|i| {
                    value
                        .bytes
                        .get(i..i + 2)
                        .and_then(|sub| u8::from_str_radix(sub, 16).ok())
                })
                .collect()
            {
                Some(bytes) => Ok(bytes),
                None => Err(Error::InvalidSyntaxField("bytes")),
            }?,
            value
                .expiry_time
                .parse::<u64>()
                .map_err(|_| Error::InvalidSyntaxField("expiryTime"))?,
        )?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexationPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    index: String, // VecPrefix<u8, BoundedU32<PREFIXED_INDEX_LENGTH_MIN, PREFIXED_INDEX_LENGTH_MAX>>,
    data: String,  // VecPrefix<u8, BoundedU32<0, PREFIXED_DATA_LENGTH_MAX>>,
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
            match (0..value.index.len())
                .step_by(2)
                .map(|i| {
                    value
                        .index
                        .get(i..i + 2)
                        .and_then(|sub| u8::from_str_radix(sub, 16).ok())
                })
                .collect()
            {
                Some(index) => Ok(index),
                None => Err(Error::InvalidSyntaxField("index")),
            }?,
            match (0..value.data.len())
                .step_by(2)
                .map(|i| {
                    value
                        .data
                        .get(i..i + 2)
                        .and_then(|sub| u8::from_str_radix(sub, 16).ok())
                })
                .collect()
            {
                Some(data) => Ok(data),
                None => Err(Error::InvalidSyntaxField("data")),
            }?,
        )?)
    }
}
