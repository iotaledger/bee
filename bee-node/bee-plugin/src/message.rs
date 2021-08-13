// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use crate::grpc::{payload::PayloadKind, DataPayload, Message, ParentsBlock, Payload};

impl From<&bee_message::Message> for Message {
    fn from(message: &bee_message::Message) -> Self {
        Self {
            parents_block: message.parents_blocks().map(ParentsBlock::from).collect(),
            issuer_public_key: message.issuer_public_key().to_vec(),
            issue_timestamp: message.issue_timestamp(),
            sequence_number: message.sequence_number(),
            payload: message.payload().as_ref().map(Payload::from),
            nonce: message.nonce(),
            signature: message.signature().to_vec(),
        }
    }
}

impl From<&bee_message::payload::Payload> for Payload {
    fn from(payload: &bee_message::payload::Payload) -> Self {
        let payload_kind = match payload {
            bee_message::payload::Payload::Data(payload) => PayloadKind::Data(DataPayload::from(payload.as_ref())),
            bee_message::payload::Payload::Transaction(_) => todo!(),
            bee_message::payload::Payload::Fpc(_) => todo!(),
            bee_message::payload::Payload::ApplicationMessage(_) => todo!(),
            bee_message::payload::Payload::Dkg(_) => todo!(),
            bee_message::payload::Payload::Beacon(_) => todo!(),
            bee_message::payload::Payload::CollectiveBeacon(_) => todo!(),
            bee_message::payload::Payload::SaltDeclaration(_) => todo!(),
            bee_message::payload::Payload::Indexation(_) => todo!(),
        };

        Self {
            payload_kind: Some(payload_kind),
        }
    }
}

impl From<&bee_message::payload::data::DataPayload> for DataPayload {
    fn from(data_payload: &bee_message::payload::data::DataPayload) -> Self {
        Self {
            data: data_payload.data().to_vec(),
        }
    }
}

impl From<&bee_message::parents::ParentsBlock> for ParentsBlock {
    fn from(_parents_block: &bee_message::parents::ParentsBlock) -> Self {
        todo!()
    }
}
