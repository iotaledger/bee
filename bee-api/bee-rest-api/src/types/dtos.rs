use crate::types::error::Error;

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
            MILESTONE_PUBLIC_KEY_LENGTH,
        },
        OptionalPayload,
        receipt::{MigratedFundsEntry, ReceiptPayload, TailTransactionHash, TAIL_TRANSACTION_HASH_LEN},
        transaction::{TransactionEssence, TransactionId, TransactionPayload},
        treasury::TreasuryTransactionPayload,
        Payload,
    },
    signature::{Ed25519Signature, SignatureUnlock},
    unlock::{ReferenceUnlock, UnlockBlock, UnlockBlocks},
    Message, MessageBuilder, MessageId,
};
use bee_protocol::types::peer::Peer;

use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

use std::convert::{TryFrom, TryInto};

pub const MESSAGE_PUBLIC_KEY_LENGTH: usize = 32;
pub const PLEDGE_ID_LENGTH: usize = 32;
pub const MESSAGE_SIGNATURE_LENGTH: usize = 64;

/// The message object that nodes gossip around in the network.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageDto {
    #[serde(rename = "parentMessageIds")]
    pub parents: Vec<Vec<String>>,
    pub issuer_public_key: [u8; MESSAGE_PUBLIC_KEY_LENGTH],
    pub issue_timestamp: u64,
    pub sequence_number: u32,
    pub payload: OptionalPayloadDto,
    pub nonce: String,
    pub signature: [u8; MESSAGE_SIGNATURE_LENGTH],
}

impl From<&Message> for MessageDto {
    fn from(value: &Message) -> Self {
        MessageDto {
            parents: value.parents_blocks().map(|b| b.iter().map(|p| p.to_string()).collect()).collect(),
            issuer_public_key: *value.issuer_public_key(),
            issue_timestamp: value.issue_timestamp(),
            sequence_number: value.sequence_number(),
            payload: OptionalPayloadDto::from(match value.payload() {
                Some(payload) => Some(PayloadDto::from(payload)),
                None => None,
            }),
            nonce: value.nonce().to_string(),
            signature: *value.signature(),
        }
    }
}

impl TryFrom<&MessageDto> for Message {
    type Error = Error;

    fn try_from(value: &MessageDto) -> Result<Self, Self::Error> {
        let mut builder = MessageBuilder::new()
            .with_parents_blocks(Parents::new(
                value
                    .parents
                    .iter()
                    .map(|m| {
                        m.parse::<MessageId>()
                            .map_err(|_| Error::InvalidSyntaxField("parentMessageIds"))
                    })
                    .collect::<Result<Vec<MessageId>, Error>>()?,
            )?)
            .with_nonce(
                value
                    .nonce
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidSyntaxField("nonce"))?,
            );
        if let Some(p) = value.payload.as_ref() {
            builder = builder.with_payload(p.try_into()?);
        }

        Ok(builder.finish()?)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum OptionalPayloadDto {
    None,
    Some(PayloadDto),
}

impl From<Option<PayloadDto>> for OptionalPayloadDto {
    fn from(option: Option<PayloadDto>) -> Self {
        match option {
            None => Self::None,
            Some(payload) => Self::Some(payload),
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
    pub timestamp: u64,
    pub access_pledge_id: [u8; PLEDGE_ID_LENGTH],
    pub consensus_pledge_id: [u8; PLEDGE_ID_LENGTH],
    pub inputs: Vec<InputDto>,
    pub outputs: Vec<OutputDto>,
    pub payload: Option<PayloadDto>,
}

impl From<&TransactionEssence> for TransactionEssenceDto {
    fn from(value: &TransactionEssence) -> Self {
        TransactionEssenceDto {
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

impl TryFrom<&TransactionEssenceDto> for TransactionEssence {
    type Error = Error;

    fn try_from(value: &TransactionEssenceDto) -> Result<Self, Self::Error> {
        let mut builder = TransactionEssence::builder();

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