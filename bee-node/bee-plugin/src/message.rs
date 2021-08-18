// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types for messages.

pub use crate::grpc::Message;

use std::convert::TryInto;

use crate::grpc::{
    address::AddressKind, input::InputKind, output::OutputKind, payload::PayloadKind, signature::SignatureKind,
    unlock_block::UnlockBlockKind, Address, ApplicationMessagePayload, AssetBalance, AssetId, BeaconPayload,
    BlsAddress, BlsSignature, CollectiveBeaconPayload, Conflict, DataPayload, DkgPayload, Ed25519Address,
    Ed25519Signature, EncryptedDeal, FpcPayload, IndexationPayload, Input, MessageId, Output, OutputId, ParentsBlock,
    ParentsKind, Payload, ReferenceUnlock, Salt, SaltDeclarationPayload, Signature, SignatureLockedAssetOutput,
    SignatureLockedSingleOutput, SignatureUnlock, Timestamp, TransactionEssence, TransactionId, TransactionPayload,
    UnlockBlock, UtxoInput,
};

impl From<&bee_message::Message> for Message {
    fn from(message: &bee_message::Message) -> Self {
        Self {
            parents_blocks: message.parents_blocks().map(ParentsBlock::from).collect(),
            issuer_public_key: message.issuer_public_key().to_vec(),
            issue_timestamp: message.issue_timestamp(),
            sequence_number: message.sequence_number(),
            payload: message.payload().as_ref().map(Payload::from),
            nonce: message.nonce(),
            signature: message.signature().to_vec(),
        }
    }
}

impl From<&bee_message::parents::ParentsBlock> for ParentsBlock {
    fn from(block: &bee_message::parents::ParentsBlock) -> Self {
        Self {
            kind: ParentsKind::from(block.parents_kind()).into(),
            references: block.iter().map(MessageId::from).collect(),
        }
    }
}

impl From<bee_message::parents::ParentsKind> for ParentsKind {
    fn from(parents_kind: bee_message::parents::ParentsKind) -> Self {
        match parents_kind {
            bee_message::parents::ParentsKind::Strong => ParentsKind::Strong,
            bee_message::parents::ParentsKind::Weak => ParentsKind::Weak,
            bee_message::parents::ParentsKind::Disliked => ParentsKind::Disliked,
            bee_message::parents::ParentsKind::Liked => ParentsKind::Liked,
        }
    }
}

impl From<&bee_message::MessageId> for MessageId {
    fn from(message_id: &bee_message::MessageId) -> Self {
        Self {
            inner: message_id.to_vec(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::MessageId> for MessageId {
    fn into(self) -> bee_message::MessageId {
        bee_message::MessageId::new(self.inner.try_into().unwrap())
    }
}

impl From<&bee_message::payload::Payload> for Payload {
    fn from(payload: &bee_message::payload::Payload) -> Self {
        let payload_kind = match payload {
            bee_message::payload::Payload::Data(payload) => PayloadKind::Data(payload.as_ref().into()),
            bee_message::payload::Payload::Transaction(payload) => {
                PayloadKind::Transaction(Box::new(payload.as_ref().into()))
            }
            bee_message::payload::Payload::Fpc(payload) => PayloadKind::Fpc(payload.as_ref().into()),
            bee_message::payload::Payload::ApplicationMessage(payload) => {
                PayloadKind::ApplicationMessage(payload.as_ref().into())
            }
            bee_message::payload::Payload::Dkg(payload) => PayloadKind::Dkg(payload.as_ref().into()),
            bee_message::payload::Payload::Beacon(payload) => PayloadKind::Beacon(payload.as_ref().into()),
            bee_message::payload::Payload::CollectiveBeacon(payload) => {
                PayloadKind::CollectiveBeacon(payload.as_ref().into())
            }
            bee_message::payload::Payload::SaltDeclaration(payload) => {
                PayloadKind::SaltDeclaration(payload.as_ref().into())
            }
            bee_message::payload::Payload::Indexation(payload) => PayloadKind::Indexation(payload.as_ref().into()),
        };

        Self {
            payload_kind: Some(payload_kind),
        }
    }
}

impl From<&bee_message::payload::data::DataPayload> for DataPayload {
    fn from(payload: &bee_message::payload::data::DataPayload) -> Self {
        Self {
            data: payload.data().to_vec(),
        }
    }
}

impl From<&bee_message::payload::transaction::TransactionPayload> for TransactionPayload {
    fn from(payload: &bee_message::payload::transaction::TransactionPayload) -> Self {
        Self {
            essence: Some(Box::new(payload.essence().into())),
            unlock_blocks: payload.unlock_blocks().iter().map(UnlockBlock::from).collect(),
        }
    }
}

impl From<&bee_message::unlock::UnlockBlock> for UnlockBlock {
    fn from(unlock: &bee_message::unlock::UnlockBlock) -> Self {
        let unlock_block_kind = match unlock {
            bee_message::unlock::UnlockBlock::Signature(unlock) => UnlockBlockKind::Signature(unlock.into()),
            bee_message::unlock::UnlockBlock::Reference(unlock) => UnlockBlockKind::Reference(unlock.into()),
        };

        Self {
            unlock_block_kind: Some(unlock_block_kind),
        }
    }
}

impl From<&bee_message::unlock::SignatureUnlock> for SignatureUnlock {
    fn from(unlock: &bee_message::unlock::SignatureUnlock) -> Self {
        Self {
            signature: Some(unlock.signature().into()),
        }
    }
}

impl From<&bee_message::signature::Signature> for Signature {
    fn from(signature: &bee_message::signature::Signature) -> Self {
        let signature_kind = match signature {
            bee_message::signature::Signature::Ed25519(signature) => SignatureKind::Ed25519(signature.into()),
            bee_message::signature::Signature::Bls(signature) => SignatureKind::Bls(signature.into()),
        };

        Self {
            signature_kind: Some(signature_kind),
        }
    }
}

impl From<&bee_message::signature::Ed25519Signature> for Ed25519Signature {
    fn from(signature: &bee_message::signature::Ed25519Signature) -> Self {
        Self {
            public_key: signature.public_key().to_vec(),
            signature: signature.signature().to_vec(),
        }
    }
}

impl From<&bee_message::signature::BlsSignature> for BlsSignature {
    fn from(signature: &bee_message::signature::BlsSignature) -> Self {
        Self {
            inner: signature.to_vec(),
        }
    }
}

impl From<&bee_message::unlock::ReferenceUnlock> for ReferenceUnlock {
    fn from(reference: &bee_message::unlock::ReferenceUnlock) -> Self {
        Self {
            index: reference.index().into(),
        }
    }
}

impl From<&bee_message::payload::transaction::TransactionEssence> for TransactionEssence {
    fn from(essence: &bee_message::payload::transaction::TransactionEssence) -> Self {
        Self {
            timestamp: essence.timestamp(),
            access_pledge_id: essence.access_pledge_id().to_vec(),
            consensus_pledge_id: essence.consensus_pledge_id().to_vec(),
            inputs: essence.inputs().iter().map(Input::from).collect(),
            outputs: essence.outputs().iter().map(Output::from).collect(),
            payload: essence.payload().as_ref().map(Payload::from).map(Box::new),
        }
    }
}

impl From<&bee_message::input::Input> for Input {
    fn from(input: &bee_message::input::Input) -> Self {
        let input_kind = match input {
            bee_message::input::Input::Utxo(input) => InputKind::Utxo(input.into()),
        };

        Self {
            input_kind: Some(input_kind),
        }
    }
}

impl From<&bee_message::input::UtxoInput> for UtxoInput {
    fn from(input: &bee_message::input::UtxoInput) -> Self {
        Self {
            output_id: Some(input.output_id().into()),
        }
    }
}

impl From<&bee_message::output::OutputId> for OutputId {
    fn from(output_id: &bee_message::output::OutputId) -> Self {
        Self {
            transaction_id: Some(output_id.transaction_id().into()),
            index: output_id.index().into(),
        }
    }
}

impl From<&bee_message::payload::transaction::TransactionId> for TransactionId {
    fn from(transaction_id: &bee_message::payload::transaction::TransactionId) -> Self {
        Self {
            inner: transaction_id.to_vec(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::transaction::TransactionId> for TransactionId {
    fn into(self) -> bee_message::payload::transaction::TransactionId {
        bee_message::payload::transaction::TransactionId::new(self.inner.try_into().unwrap())
    }
}

impl From<&bee_message::output::Output> for Output {
    fn from(output: &bee_message::output::Output) -> Self {
        let output_kind = match output {
            bee_message::output::Output::SignatureLockedSingle(output) => {
                OutputKind::SignatureLockedSingle(output.into())
            }
            bee_message::output::Output::SignatureLockedAsset(output) => {
                OutputKind::SignatureLockedAsset(output.into())
            }
        };

        Self {
            output_kind: Some(output_kind),
        }
    }
}

impl From<&bee_message::output::SignatureLockedSingleOutput> for SignatureLockedSingleOutput {
    fn from(output: &bee_message::output::SignatureLockedSingleOutput) -> Self {
        Self {
            address: Some(output.address().into()),
            amount: output.amount(),
        }
    }
}

impl From<&bee_message::address::Address> for Address {
    fn from(address: &bee_message::address::Address) -> Self {
        let address_kind = match address {
            bee_message::address::Address::Ed25519(address) => AddressKind::Ed25519(address.into()),
            bee_message::address::Address::Bls(address) => AddressKind::Bls(address.into()),
        };

        Self {
            address_kind: Some(address_kind),
        }
    }
}

impl From<&bee_message::address::Ed25519Address> for Ed25519Address {
    fn from(address: &bee_message::address::Ed25519Address) -> Self {
        Self {
            inner: address.to_vec(),
        }
    }
}

impl From<&bee_message::address::BlsAddress> for BlsAddress {
    fn from(address: &bee_message::address::BlsAddress) -> Self {
        Self {
            inner: address.to_vec(),
        }
    }
}

impl From<&bee_message::output::SignatureLockedAssetOutput> for SignatureLockedAssetOutput {
    fn from(output: &bee_message::output::SignatureLockedAssetOutput) -> Self {
        Self {
            address: Some(output.address().into()),
            balances: output.balance_iter().map(AssetBalance::from).collect(),
        }
    }
}

impl From<&bee_message::output::AssetBalance> for AssetBalance {
    fn from(balance: &bee_message::output::AssetBalance) -> Self {
        Self {
            id: Some(balance.id().into()),
            balance: balance.balance(),
        }
    }
}

impl From<&bee_message::output::AssetId> for AssetId {
    fn from(asset_id: &bee_message::output::AssetId) -> Self {
        Self {
            inner: asset_id.to_vec(),
        }
    }
}

impl From<&bee_message::payload::fpc::FpcPayload> for FpcPayload {
    fn from(payload: &bee_message::payload::fpc::FpcPayload) -> Self {
        Self {
            conflicts: payload.conflicts().map(Conflict::from).collect(),
            timestamps: payload.timestamps().map(Timestamp::from).collect(),
        }
    }
}


#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::fpc::FpcPayload> for FpcPayload {
    fn into(self) -> bee_message::payload::fpc::FpcPayload {
        bee_message::payload::fpc::FpcPayloadBuilder::default().with_conflicts(conflicts)
    }
}

impl From<&bee_message::payload::fpc::Conflict> for Conflict {
    fn from(conflict: &bee_message::payload::fpc::Conflict) -> Self {
        Self {
            transaction_id: Some(conflict.transaction_id().into()),
            opinion: conflict.opinion().into(),
            round: conflict.round().into(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::fpc::Conflict> for Conflict {
    fn into(self) -> bee_message::payload::fpc::Conflict {
        bee_message::payload::fpc::Conflict::new(
            self.transaction_id.unwrap().into(),
            self.opinion.try_into().unwrap(),
            self.round.try_into().unwrap(),
        )
    }
}

impl From<&bee_message::payload::fpc::Timestamp> for Timestamp {
    fn from(timestamp: &bee_message::payload::fpc::Timestamp) -> Self {
        Self {
            message_id: Some(timestamp.message_id().into()),
            opinion: timestamp.opinion().into(),
            round: timestamp.round().into(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::fpc::Timestamp> for Timestamp {
    fn into(self) -> bee_message::payload::fpc::Timestamp {
        bee_message::payload::fpc::Timestamp::new(
            self.message_id.unwrap().into(),
            self.opinion.try_into().unwrap(),
            self.round.try_into().unwrap(),
        )
    }
}

impl From<&bee_message::payload::drng::ApplicationMessagePayload> for ApplicationMessagePayload {
    fn from(payload: &bee_message::payload::drng::ApplicationMessagePayload) -> Self {
        Self {
            instance_id: payload.instance_id(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::drng::ApplicationMessagePayload> for ApplicationMessagePayload {
    fn into(self) -> bee_message::payload::drng::ApplicationMessagePayload {
        bee_message::payload::drng::ApplicationMessagePayload::new(self.instance_id)
    }
}

impl From<&bee_message::payload::drng::DkgPayload> for DkgPayload {
    fn from(payload: &bee_message::payload::drng::DkgPayload) -> Self {
        Self {
            instance_id: payload.instance_id(),
            from_index: payload.from_index(),
            to_index: payload.to_index(),
            deal: Some(payload.deal().into()),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::drng::DkgPayload> for DkgPayload {
    fn into(self) -> bee_message::payload::drng::DkgPayload {
        bee_message::payload::drng::DkgPayloadBuilder::default()
            .with_instance_id(self.instance_id)
            .with_from_index(self.from_index)
            .with_to_index(self.to_index)
            .with_deal(self.deal.unwrap().into())
            .finish()
            .unwrap()
    }
}

impl From<&bee_message::payload::drng::EncryptedDeal> for EncryptedDeal {
    fn from(encrypted_deal: &bee_message::payload::drng::EncryptedDeal) -> Self {
        Self {
            dh_key: encrypted_deal.dh_key().to_vec(),
            nonce: encrypted_deal.nonce().to_vec(),
            encrpyted_share: encrypted_deal.encrypted_share().to_vec(),
            threshold: encrypted_deal.threshold(),
            commitments: encrypted_deal.commitments().to_vec(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::drng::EncryptedDeal> for EncryptedDeal {
    fn into(self) -> bee_message::payload::drng::EncryptedDeal {
        bee_message::payload::drng::EncryptedDealBuilder::default()
            .with_dh_key(self.dh_key)
            .with_nonce(self.nonce)
            .with_encrypted_share(self.encrpyted_share)
            .with_threshold(self.threshold)
            .with_commitments(self.commitments)
            .finish()
            .unwrap()
    }
}

impl From<&bee_message::payload::drng::BeaconPayload> for BeaconPayload {
    fn from(payload: &bee_message::payload::drng::BeaconPayload) -> Self {
        Self {
            instance_id: payload.instance_id(),
            round: payload.round(),
            partial_public_key: payload.partial_public_key().to_vec(),
            partial_signature: payload.partial_signature().to_vec(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::drng::BeaconPayload> for BeaconPayload {
    fn into(self) -> bee_message::payload::drng::BeaconPayload {
        bee_message::payload::drng::BeaconPayloadBuilder::default()
            .with_instance_id(self.instance_id)
            .with_round(self.round)
            .with_partial_public_key(self.partial_public_key.try_into().unwrap())
            .with_partial_signature(self.partial_signature.try_into().unwrap())
            .finish()
            .unwrap()
    }
}

impl From<&bee_message::payload::drng::CollectiveBeaconPayload> for CollectiveBeaconPayload {
    fn from(payload: &bee_message::payload::drng::CollectiveBeaconPayload) -> Self {
        Self {
            instance_id: payload.instance_id(),
            round: payload.round(),
            prev_signature: payload.prev_signature().to_vec(),
            signature: payload.signature().to_vec(),
            distributed_public_key: payload.distributed_public_key().to_vec(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::drng::CollectiveBeaconPayload> for CollectiveBeaconPayload {
    fn into(self) -> bee_message::payload::drng::CollectiveBeaconPayload {
        bee_message::payload::drng::CollectiveBeaconPayloadBuilder::default()
            .with_instance_id(self.instance_id)
            .with_round(self.round)
            .with_prev_signature(self.prev_signature.try_into().unwrap())
            .with_signature(self.signature.try_into().unwrap())
            .with_distributed_public_key(self.distributed_public_key.try_into().unwrap())
            .finish()
            .unwrap()
    }
}

impl From<&bee_message::payload::salt_declaration::SaltDeclarationPayload> for SaltDeclarationPayload {
    fn from(payload: &bee_message::payload::salt_declaration::SaltDeclarationPayload) -> Self {
        Self {
            node_id: payload.node_id(),
            salt: Some(payload.salt().into()),
            timestamp: payload.timestamp(),
            signature: payload.signature().to_vec(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::salt_declaration::SaltDeclarationPayload> for SaltDeclarationPayload {
    fn into(self) -> bee_message::payload::salt_declaration::SaltDeclarationPayload {
        bee_message::payload::salt_declaration::SaltDeclarationPayloadBuilder::new()
            .with_salt(self.salt.unwrap().into())
            .with_node_id(self.node_id)
            .with_timestamp(self.timestamp)
            .with_signature(self.signature.try_into().unwrap())
            .finish()
            .unwrap()
    }
}

impl From<&bee_message::payload::salt_declaration::Salt> for Salt {
    fn from(salt: &bee_message::payload::salt_declaration::Salt) -> Self {
        Self {
            bytes: salt.bytes().to_vec(),
            expiry_time: salt.expiry_time(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::salt_declaration::Salt> for Salt {
    fn into(self) -> bee_message::payload::salt_declaration::Salt {
        bee_message::payload::salt_declaration::Salt::new(self.bytes, self.expiry_time).unwrap()
    }
}

impl From<&bee_message::payload::indexation::IndexationPayload> for IndexationPayload {
    fn from(payload: &bee_message::payload::indexation::IndexationPayload) -> Self {
        Self {
            index: payload.index().to_vec(),
            data: payload.data().to_vec(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::indexation::IndexationPayload> for IndexationPayload {
    fn into(self) -> bee_message::payload::indexation::IndexationPayload {
        bee_message::payload::indexation::IndexationPayload::new(self.index, self.data).unwrap()
    }
}
