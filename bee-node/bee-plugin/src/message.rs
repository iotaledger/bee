// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types for messages.

pub use crate::grpc::Message;

use std::convert::TryInto;

use crate::grpc::{
    address::Kind as AddressKind, input::Kind as InputKind, output::Kind as OutputKind, payload::Kind as PayloadKind,
    signature::Kind as SignatureKind, unlock_block::Kind as UnlockBlockKind, Address, ApplicationMessagePayload,
    AssetBalance, AssetId, BeaconPayload, BlsAddress, BlsSignature, CollectiveBeaconPayload, Conflict, DataPayload,
    DkgPayload, Ed25519Address, Ed25519Signature, EncryptedDeal, FpcPayload, IndexationPayload, Input, MessageId,
    Output, OutputId, ParentsBlock, ParentsKind, Payload, ReferenceUnlock, Salt, SaltDeclarationPayload, Signature,
    SignatureLockedAssetOutput, SignatureLockedSingleOutput, SignatureUnlock, Timestamp, TransactionEssence,
    TransactionId, TransactionPayload, UnlockBlock, UtxoInput,
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::Message> for Message {
    fn into(self) -> bee_message::Message {
        let mut builder = bee_message::MessageBuilder::default()
            .with_issuer_public_key(self.issuer_public_key.try_into().unwrap())
            .with_issue_timestamp(self.issue_timestamp)
            .with_sequence_number(self.sequence_number)
            .with_nonce(self.nonce)
            .with_signature(self.signature.try_into().unwrap());

        if let Some(payload) = self.payload {
            builder = builder.with_payload(payload.into());
        }

        for parents_block in self.parents_blocks {
            builder = builder.add_parents_block(parents_block.into());
        }

        builder.finish().unwrap()
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::parents::ParentsBlock> for ParentsBlock {
    fn into(self) -> bee_message::parents::ParentsBlock {
        bee_message::parents::ParentsBlock::new(
            ParentsKind::from_i32(self.kind).unwrap().into(),
            self.references.into_iter().map(MessageId::into).collect(),
        )
        .unwrap()
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::parents::ParentsKind> for ParentsKind {
    fn into(self) -> bee_message::parents::ParentsKind {
        match self {
            ParentsKind::Strong => bee_message::parents::ParentsKind::Strong,
            ParentsKind::Weak => bee_message::parents::ParentsKind::Weak,
            ParentsKind::Disliked => bee_message::parents::ParentsKind::Disliked,
            ParentsKind::Liked => bee_message::parents::ParentsKind::Liked,
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
        let kind = match payload {
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

        Self { kind: Some(kind) }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::Payload> for Payload {
    fn into(self) -> bee_message::payload::Payload {
        match self.kind.unwrap() {
            PayloadKind::Data(payload) => bee_message::payload::Payload::Data(Box::new(payload.into())),
            PayloadKind::Transaction(payload) => {
                bee_message::payload::Payload::Transaction(Box::new((*payload).into()))
            }
            PayloadKind::Fpc(payload) => bee_message::payload::Payload::Fpc(Box::new(payload.into())),
            PayloadKind::ApplicationMessage(payload) => {
                bee_message::payload::Payload::ApplicationMessage(Box::new(payload.into()))
            }
            PayloadKind::Dkg(payload) => bee_message::payload::Payload::Dkg(Box::new(payload.into())),
            PayloadKind::Beacon(payload) => bee_message::payload::Payload::Beacon(Box::new(payload.into())),
            PayloadKind::CollectiveBeacon(payload) => {
                bee_message::payload::Payload::CollectiveBeacon(Box::new(payload.into()))
            }
            PayloadKind::SaltDeclaration(payload) => {
                bee_message::payload::Payload::SaltDeclaration(Box::new(payload.into()))
            }
            PayloadKind::Indexation(payload) => bee_message::payload::Payload::Indexation(Box::new(payload.into())),
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::data::DataPayload> for DataPayload {
    fn into(self) -> bee_message::payload::data::DataPayload {
        bee_message::payload::data::DataPayload::new(self.data).unwrap()
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::transaction::TransactionPayload> for TransactionPayload {
    fn into(self) -> bee_message::payload::transaction::TransactionPayload {
        bee_message::payload::transaction::TransactionPayloadBuilder::default()
            .with_essence((*self.essence.unwrap()).into())
            .with_unlock_blocks(
                bee_message::unlock::UnlockBlocks::new(self.unlock_blocks.into_iter().map(UnlockBlock::into).collect())
                    .unwrap(),
            )
            .finish()
            .unwrap()
    }
}

impl From<&bee_message::unlock::UnlockBlock> for UnlockBlock {
    fn from(block: &bee_message::unlock::UnlockBlock) -> Self {
        let kind = match block {
            bee_message::unlock::UnlockBlock::Signature(block) => UnlockBlockKind::Signature(block.into()),
            bee_message::unlock::UnlockBlock::Reference(block) => UnlockBlockKind::Reference(block.into()),
        };

        Self { kind: Some(kind) }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::unlock::UnlockBlock> for UnlockBlock {
    fn into(self) -> bee_message::unlock::UnlockBlock {
        match self.kind.unwrap() {
            UnlockBlockKind::Signature(block) => bee_message::unlock::UnlockBlock::Signature(block.into()),
            UnlockBlockKind::Reference(block) => bee_message::unlock::UnlockBlock::Reference(block.into()),
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::unlock::SignatureUnlock> for SignatureUnlock {
    fn into(self) -> bee_message::unlock::SignatureUnlock {
        bee_message::unlock::SignatureUnlock::new(self.signature.unwrap().into())
    }
}

impl From<&bee_message::signature::Signature> for Signature {
    fn from(signature: &bee_message::signature::Signature) -> Self {
        let kind = match signature {
            bee_message::signature::Signature::Ed25519(signature) => SignatureKind::Ed25519(signature.into()),
            bee_message::signature::Signature::Bls(signature) => SignatureKind::Bls(signature.into()),
        };

        Self { kind: Some(kind) }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::signature::Signature> for Signature {
    fn into(self) -> bee_message::signature::Signature {
        match self.kind.unwrap() {
            SignatureKind::Ed25519(signature) => bee_message::signature::Signature::Ed25519(signature.into()),
            SignatureKind::Bls(signature) => bee_message::signature::Signature::Bls(signature.into()),
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::signature::Ed25519Signature> for Ed25519Signature {
    fn into(self) -> bee_message::signature::Ed25519Signature {
        bee_message::signature::Ed25519Signature::new(
            self.public_key.try_into().unwrap(),
            self.signature.try_into().unwrap(),
        )
    }
}

impl From<&bee_message::signature::BlsSignature> for BlsSignature {
    fn from(signature: &bee_message::signature::BlsSignature) -> Self {
        Self {
            inner: signature.to_vec(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::signature::BlsSignature> for BlsSignature {
    fn into(self) -> bee_message::signature::BlsSignature {
        bee_message::signature::BlsSignature::new(self.inner.try_into().unwrap())
    }
}

impl From<&bee_message::unlock::ReferenceUnlock> for ReferenceUnlock {
    fn from(reference: &bee_message::unlock::ReferenceUnlock) -> Self {
        Self {
            index: reference.index().into(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::unlock::ReferenceUnlock> for ReferenceUnlock {
    fn into(self) -> bee_message::unlock::ReferenceUnlock {
        bee_message::unlock::ReferenceUnlock::new(self.index.try_into().unwrap()).unwrap()
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::payload::transaction::TransactionEssence> for TransactionEssence {
    fn into(self) -> bee_message::payload::transaction::TransactionEssence {
        let mut builder = bee_message::payload::transaction::TransactionEssenceBuilder::default()
            .with_timestamp(self.timestamp)
            .with_access_pledge_id(self.access_pledge_id.try_into().unwrap())
            .with_consensus_pledge_id(self.consensus_pledge_id.try_into().unwrap())
            .with_inputs(self.inputs.into_iter().map(Input::into).collect())
            .with_outputs(self.outputs.into_iter().map(Output::into).collect());

        if let Some(payload) = self.payload {
            builder = builder.with_payload((*payload).into());
        }

        builder.finish().unwrap()
    }
}

impl From<&bee_message::input::Input> for Input {
    fn from(input: &bee_message::input::Input) -> Self {
        let kind = match input {
            bee_message::input::Input::Utxo(input) => InputKind::Utxo(input.into()),
        };

        Self { kind: Some(kind) }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::input::Input> for Input {
    fn into(self) -> bee_message::input::Input {
        match self.kind.unwrap() {
            InputKind::Utxo(input) => bee_message::input::Input::Utxo(input.into()),
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::input::UtxoInput> for UtxoInput {
    fn into(self) -> bee_message::input::UtxoInput {
        bee_message::input::UtxoInput::new(self.output_id.unwrap().into())
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::output::OutputId> for OutputId {
    fn into(self) -> bee_message::output::OutputId {
        bee_message::output::OutputId::new(self.transaction_id.unwrap().into(), self.index.try_into().unwrap()).unwrap()
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
        let kind = match output {
            bee_message::output::Output::SignatureLockedSingle(output) => {
                OutputKind::SignatureLockedSingle(output.into())
            }
            bee_message::output::Output::SignatureLockedAsset(output) => {
                OutputKind::SignatureLockedAsset(output.into())
            }
        };

        Self { kind: Some(kind) }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::output::Output> for Output {
    fn into(self) -> bee_message::output::Output {
        match self.kind.unwrap() {
            OutputKind::SignatureLockedSingle(output) => {
                bee_message::output::Output::SignatureLockedSingle(output.into())
            }
            OutputKind::SignatureLockedAsset(output) => {
                bee_message::output::Output::SignatureLockedAsset(output.into())
            }
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::output::SignatureLockedSingleOutput> for SignatureLockedSingleOutput {
    fn into(self) -> bee_message::output::SignatureLockedSingleOutput {
        bee_message::output::SignatureLockedSingleOutput::new(self.address.unwrap().into(), self.amount).unwrap()
    }
}

impl From<&bee_message::address::Address> for Address {
    fn from(address: &bee_message::address::Address) -> Self {
        let kind = match address {
            bee_message::address::Address::Ed25519(address) => AddressKind::Ed25519(address.into()),
            bee_message::address::Address::Bls(address) => AddressKind::Bls(address.into()),
        };

        Self { kind: Some(kind) }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::address::Address> for Address {
    fn into(self) -> bee_message::address::Address {
        match self.kind.unwrap() {
            AddressKind::Ed25519(address) => bee_message::address::Address::Ed25519(address.into()),
            AddressKind::Bls(address) => bee_message::address::Address::Bls(address.into()),
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::address::Ed25519Address> for Ed25519Address {
    fn into(self) -> bee_message::address::Ed25519Address {
        bee_message::address::Ed25519Address::new(self.inner.try_into().unwrap())
    }
}

impl From<&bee_message::address::BlsAddress> for BlsAddress {
    fn from(address: &bee_message::address::BlsAddress) -> Self {
        Self {
            inner: address.to_vec(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::address::BlsAddress> for BlsAddress {
    fn into(self) -> bee_message::address::BlsAddress {
        bee_message::address::BlsAddress::new(self.inner.try_into().unwrap())
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::output::SignatureLockedAssetOutput> for SignatureLockedAssetOutput {
    fn into(self) -> bee_message::output::SignatureLockedAssetOutput {
        bee_message::output::SignatureLockedAssetOutput::new(
            self.address.unwrap().into(),
            self.balances.into_iter().map(AssetBalance::into).collect(),
        )
        .unwrap()
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

#[allow(clippy::from_over_into)]
impl Into<bee_message::output::AssetBalance> for AssetBalance {
    fn into(self) -> bee_message::output::AssetBalance {
        bee_message::output::AssetBalance::new(self.id.unwrap().into(), self.balance)
    }
}

impl From<&bee_message::output::AssetId> for AssetId {
    fn from(asset_id: &bee_message::output::AssetId) -> Self {
        Self {
            inner: asset_id.to_vec(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bee_message::output::AssetId> for AssetId {
    fn into(self) -> bee_message::output::AssetId {
        bee_message::output::AssetId::new(self.inner.try_into().unwrap())
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
        bee_message::payload::fpc::FpcPayloadBuilder::default()
            .with_conflicts(self.conflicts.into_iter().map(Conflict::into).collect())
            .with_timestamps(self.timestamps.into_iter().map(Timestamp::into).collect())
            .finish()
            .unwrap()
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
