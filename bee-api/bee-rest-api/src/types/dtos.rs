use crate::types::error::Error;

use bee_message::{
    MESSAGE_PUBLIC_KEY_LENGTH,
    MESSAGE_SIGNATURE_LENGTH,
    PREFIXED_PARENTS_LENGTH_MIN,
    PREFIXED_PARENTS_LENGTH_MAX,
    address::{Address, Ed25519Address, BlsAddress},
    error::ValidationError,
    input::{Input, TreasuryInput, UtxoInput},
    milestone::MilestoneIndex,
    output::{Output, SignatureLockedSingleOutput, SignatureLockedAssetOutput, AssetBalance, AssetId, OUTPUT_INDEX_MAX, OutputId},
    parents::{ParentsBlock, ParentsKind},
    payload::{
        indexation::IndexationPayload,
        milestone::{
            MilestoneId, MilestonePayload, MilestonePayloadEssence, MILESTONE_MERKLE_PROOF_LENGTH,
            MILESTONE_PUBLIC_KEY_LENGTH,
        },
        OptionalPayload,
        receipt::{MigratedFundsEntry, ReceiptPayload, TailTransactionHash, TAIL_TRANSACTION_HASH_LEN},
        transaction::{TransactionEssence, TransactionId, TransactionPayload, PLEDGE_ID_LENGTH},
        treasury::TreasuryTransactionPayload,
        Payload,
    },
    signature::{Ed25519Signature, Signature, BlsSignature, SignatureUnlock},
    unlock::{ReferenceUnlock, UnlockBlock, UnlockBlocks},
    Message, MessageBuilder, MessageId,
};
use bee_protocol::types::peer::Peer;
use bee_packable::{BoundedU8, VecPrefix, InvalidBoundedU8};

use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

use std::str::FromStr;
use std::ops::Deref;
use std::convert::{TryFrom, TryInto};

/// The message object that nodes gossip around in the network.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MessageDto {
    pub parents: Vec<ParentsBlockDto>,
    pub issuer_public_key: String,//[u8; MESSAGE_PUBLIC_KEY_LENGTH],
    pub issue_timestamp: String,
    pub sequence_number: String,
    pub payload: Option<PayloadDto>,
    pub nonce: String,
    #[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))]
    pub signature: String,//[u8; MESSAGE_SIGNATURE_LENGTH],
}

impl From<&Message> for MessageDto {
    fn from(value: &Message) -> Self {
        MessageDto {
            parents: value.parents_blocks().map(|b| b.into()).collect(),
            issuer_public_key: hex::encode(value.issuer_public_key()),
            issue_timestamp: value.issue_timestamp().to_string(),
            sequence_number: value.sequence_number().to_string(),
            payload: match value.payload() {
                Some(payload) => Some(PayloadDto::from(payload)),
                None => None,
            },
            nonce: value.nonce().to_string(),
            signature: hex::encode(value.signature()),
        }
    }
}

impl TryFrom<&MessageDto> for Message {
    type Error = Error;

    fn try_from(value: &MessageDto) -> Result<Self, Self::Error> {
        let mut builder = MessageBuilder::new()
            .with_parents_blocks(
                value
                    .parents
                    .iter()
                    .map(|p| ParentsBlock::try_from(p))
                    .collect::<Result<Vec<ParentsBlock>, Error>>()?,
            )
            .with_issuer_public_key({
                    let mut public_key: [u8; MESSAGE_PUBLIC_KEY_LENGTH];

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
                let mut signature: [u8; MESSAGE_SIGNATURE_LENGTH];

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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ParentsBlockDto {
    kind: u8,
    references: Vec<String>,
}

impl From<&ParentsBlock> for ParentsBlockDto {
    fn from(value: &ParentsBlock) -> Self {
        ParentsBlockDto {
            kind: value.parents_kind() as u8,
            references: value.iter().map(|p| p.to_string()).collect::<Vec<String>>(),
        } 
    }
}

impl TryFrom<&ParentsBlockDto> for ParentsBlock {
    type Error = Error;

    fn try_from(value: &ParentsBlockDto) -> Result<Self, Self::Error> {
        let references = value
            .references
            .iter()
            .map(|m| {
                m.parse::<MessageId>()
                    .map_err(|_| Error::InvalidSyntaxField("parentMessageIds"))
        })
        .collect::<Result<Vec<MessageId>, Error>>()?;
        match ParentsBlock::new( ParentsKind::try_from(value.kind)?, references) {
            Ok(parents_blocks) => Ok(parents_blocks),
            Err(e) => Err(Error::Message(e)),
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
            Payload::ApplicationMessage(a) => PayloadDto::ApplicationMessage(Box::new(ApplicationMessagePayloadDto::from(a.as_ref()))),
            Payload::Dkg(d) => PayloadDto::Dkg(Box::new(DkgPayloadDto::from(d.as_ref()))),
            Payload::Beacon(b) => PayloadDto::Beacon(Box::new(BeaconPayloadDto::from(b.as_ref()))),
            Payload::CollectiveBeacon(c) => PayloadDto::CollectiveBeacon(Box::new(CollectiveBeaconPayloadDto::from(c.as_ref()))),
            Payload::SaltDeclaration(s) => PayloadDto::SaltDeclaration(Box::new(SaltDeclarationPayloadDto::from(s.as_ref()))),
            Payload::Indexation(i) => PayloadDto::Indexation(Box::new(IndexationPayloadDto::from(i.as_ref())))
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
            PayloadDto::ApplicationMessage(a) => Payload::ApplicationMessage(Box::new(ApplicationMessagePayload::try_from(a.as_ref())?)),
            PayloadDto::Dkg(d) => Payload::Dkg(Box::new(DkgPayload::try_from(d.as_ref())?)),
            PayloadDto::Beacon(b) => Payload::Beacon(Box::new(BeaconPayload::try_from(b.as_ref())?)),
            PayloadDto::CollectiveBeacon(c) => Payload::CollectiveBeacon(Box::new(CollectiveBeaconPayload::try_from(c.as_ref())?)),
            PayloadDto::SaltDeclaration(s) => Payload::SaltDeclaration(Box::new(SaltDeclarationPayload::try_from(s.as_ref())?)),
            PayloadDto::Indexation(i) => Payload::Indexation(Box::new(IndexationPayload::try_from(i.as_ref())?))
        })
    }
}

/// The payload type to define a value transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionPayloadDto {
    pub transaction_essence: TransactionEssenceDto,
    #[serde(rename = "unlockBlocks")]
    pub unlock_blocks: Vec<UnlockBlockDto>,
}

impl From<&TransactionPayload> for TransactionPayloadDto {
    fn from(value: &TransactionPayload) -> Self {
        TransactionPayloadDto {
            transaction_essence: value.essence().into(),
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
            .with_essence((&value.transaction_essence).try_into()?)
            .with_unlock_blocks(UnlockBlocks::new(unlock_blocks)?);

        Ok(builder.finish()?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionEssenceDto {
    pub timestamp: String,
    pub access_pledge_id: String,//[u8; PLEDGE_ID_LENGTH],
    pub consensus_pledge_id: String,//[u8; PLEDGE_ID_LENGTH],
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
            inputs: value.inputs().iter().map(|i| i.into()).collect::<Vec<_>>(),
            outputs: value.outputs().iter().map(|o| o.into()).collect::<Vec<_>>(),
            payload: match value.payload() {
                    Some(p) => Some(PayloadDto::from(p)),
                    None => None,
            },
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
                    let mut access_pledge_id: [u8; PLEDGE_ID_LENGTH];

                    hex::decode_to_slice(&value.access_pledge_id, &mut access_pledge_id)
                        .map_err(|_| Error::InvalidSyntaxField("accessPledgeId"))?;
                    
                    access_pledge_id
            })
            .with_consensus_pledge_id({
                    let mut consensus_pledge_id: [u8; PLEDGE_ID_LENGTH];

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
            builder = builder.with_payload(
                Payload::try_from(p)
                .map_err(|_| Error::InvalidSemanticField("payload"))?,
            );
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
        Ok(SignatureUnlock(value.0.try_into()?))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferenceUnlockDto (u16);

impl From<&ReferenceUnlock> for ReferenceUnlockDto {
    fn from(value: &ReferenceUnlock) -> Self {
        ReferenceUnlockDto(value.index())
    }
}

impl TryFrom<&ReferenceUnlockDto> for ReferenceUnlock {
    type Error = Error;

    fn try_from(value: &ReferenceUnlockDto) -> Result<Self, Self::Error> {
        value.0.try_into()
        .map_err(|_| Error::InvalidSemanticField("index"))
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
    public_key: String,//[u8; Self::PUBLIC_KEY_LENGTH],
    signature: String,//[u8; Self::SIGNATURE_LENGTH],
}

impl From<&Ed25519Signature> for Ed25519SignatureDto {
    fn from(value: &Ed25519Signature) -> Self {
        Ed25519SignatureDto {
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
                let mut public_key: [u8; Ed25519Signature::PUBLIC_KEY_LENGTH];

                hex::decode_to_slice(&value.public_key, &mut public_key)
                    .map_err(|_| Error::InvalidSyntaxField("publicKey"))?;
                    
                public_key
            },
            {
                let mut signature: [u8; Ed25519Signature::SIGNATURE_LENGTH];

                hex::decode_to_slice(&value.signature, &mut signature)
                    .map_err(|_| Error::InvalidSyntaxField("publicKey"))?;
                    
                signature
            }
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
            let mut bytes: [u8; BlsSignature::LENGTH];

            hex::decode_to_slice(&value.0, &mut bytes)
                .map_err(|_| Error::InvalidSyntaxField("bytes"))?;
                    
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
                i.transaction_output_index
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
            Output::SignatureLockedAsset(s) => OutputDto::SignatureLockedAsset( SignatureLockedAssetOutputDto {
                address: s.address().into(),
                balances: s.balance_iter().map(|b| AssetBalanceDto {
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
                        let mut asset_id: [u8; AssetId::LENGTH];

                        match hex::decode_to_slice(&b.id, &mut asset_id)
                            .map_err(|_| Error::InvalidSyntaxField("assetId")) {
                                Ok(()) => (),
                                Err(e) => return Err(e),
                        }
                    
                        Ok(AssetBalance::new(asset_id.into(), b.balance))
                    })
                    .collect::<Result<Vec<AssetBalance>, _>>()?
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
            AddressDto::Bls(b) => Ok(Address::Bls(BlsAddress::from_str(&b.0)?))
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