// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Fetch access operations.

use crate::{storage::Storage, trees::*};

use bee_common::packable::Packable;
use bee_ledger::types::{
    snapshot::info::SnapshotInfo, Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt,
    TreasuryOutput,
};
use bee_message::{
    address::{Address, Ed25519Address, ED25519_ADDRESS_LENGTH},
    milestone::{Milestone, MilestoneIndex},
    output::{OutputId, OUTPUT_ID_LENGTH},
    payload::indexation::{PaddedIndex, INDEXATION_PADDED_INDEX_LENGTH},
    Message, MessageId, MESSAGE_ID_LENGTH,
};
use bee_storage::{access::Fetch, backend::StorageBackend, system::System};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

use std::convert::{TryFrom, TryInto};

#[async_trait::async_trait]
impl Fetch<u8, System> for Storage {
    async fn fetch(&self, &key: &u8) -> Result<Option<System>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get(&[key])?
            // Unpacking from storage is fine.
            .map(|v| System::unpack_unchecked(&mut v.as_ref()).unwrap()))
    }
}

#[async_trait::async_trait]
impl Fetch<MessageId, Message> for Storage {
    async fn fetch(&self, message_id: &MessageId) -> Result<Option<Message>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MESSAGE_ID_TO_MESSAGE)?
            .get(message_id)?
            // Unpacking from storage is fine.
            .map(|v| Message::unpack_unchecked(&mut v.as_ref()).unwrap()))
    }
}

#[async_trait::async_trait]
impl Fetch<MessageId, MessageMetadata> for Storage {
    async fn fetch(&self, message_id: &MessageId) -> Result<Option<MessageMetadata>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MESSAGE_ID_TO_METADATA)?
            .get(message_id)?
            // Unpacking from storage is fine.
            .map(|v| MessageMetadata::unpack_unchecked(&mut v.as_ref()).unwrap()))
    }
}

#[async_trait::async_trait]
impl Fetch<MessageId, Vec<MessageId>> for Storage {
    async fn fetch(&self, parent: &MessageId) -> Result<Option<Vec<MessageId>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .open_tree(TREE_MESSAGE_ID_TO_MESSAGE_ID)?
                .scan_prefix(parent)
                .map(|result| {
                    let (key, _) = result?;
                    let (_, child) = key.split_at(MESSAGE_ID_LENGTH);
                    // Unpacking from storage is fine.
                    let child: [u8; MESSAGE_ID_LENGTH] = child.try_into().unwrap();
                    Ok(MessageId::from(child))
                })
                .take(self.config.storage.fetch_edge_limit)
                .collect::<Result<Vec<MessageId>, Self::Error>>()?,
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<PaddedIndex, Vec<MessageId>> for Storage {
    async fn fetch(&self, index: &PaddedIndex) -> Result<Option<Vec<MessageId>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .open_tree(TREE_INDEX_TO_MESSAGE_ID)?
                .scan_prefix(index)
                .map(|result| {
                    let (key, _) = result?;
                    let (_, message_id) = key.split_at(INDEXATION_PADDED_INDEX_LENGTH);
                    // Unpacking from storage is fine.
                    let message_id: [u8; MESSAGE_ID_LENGTH] = message_id.try_into().unwrap();
                    Ok(MessageId::from(message_id))
                })
                .take(self.config.storage.fetch_index_limit)
                .collect::<Result<Vec<MessageId>, Self::Error>>()?,
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<OutputId, CreatedOutput> for Storage {
    async fn fetch(&self, output_id: &OutputId) -> Result<Option<CreatedOutput>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_TO_CREATED_OUTPUT)?
            .get(output_id.pack_new())?
            // Unpacking from storage is fine.
            .map(|v| CreatedOutput::unpack_unchecked(&mut v.as_ref()).unwrap()))
    }
}

#[async_trait::async_trait]
impl Fetch<OutputId, ConsumedOutput> for Storage {
    async fn fetch(&self, output_id: &OutputId) -> Result<Option<ConsumedOutput>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT)?
            .get(output_id.pack_new())?
            // Unpacking from storage is fine.
            .map(|v| ConsumedOutput::unpack_unchecked(&mut v.as_ref()).unwrap()))
    }
}

#[async_trait::async_trait]
impl Fetch<Ed25519Address, Vec<OutputId>> for Storage {
    async fn fetch(&self, address: &Ed25519Address) -> Result<Option<Vec<OutputId>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .open_tree(TREE_ED25519_ADDRESS_TO_OUTPUT_ID)?
                .scan_prefix(address)
                .map(|result| {
                    let (key, _) = result?;
                    let (_, output_id) = key.split_at(ED25519_ADDRESS_LENGTH);
                    // Unpacking from storage is fine.
                    Ok((<[u8; OUTPUT_ID_LENGTH]>::try_from(output_id).unwrap())
                        .try_into()
                        .unwrap())
                })
                .take(self.config.storage.fetch_output_id_limit)
                .collect::<Result<Vec<OutputId>, Self::Error>>()?,
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<(), LedgerIndex> for Storage {
    async fn fetch(&self, (): &()) -> Result<Option<LedgerIndex>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_LEDGER_INDEX)?
            .get([0x00u8])?
            // Unpacking from storage is fine.
            .map(|v| LedgerIndex::unpack_unchecked(&mut v.as_ref()).unwrap()))
    }
}

#[async_trait::async_trait]
impl Fetch<MilestoneIndex, Milestone> for Storage {
    async fn fetch(&self, index: &MilestoneIndex) -> Result<Option<Milestone>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_MILESTONE)?
            .get(index.pack_new())?
            // Unpacking from storage is fine.
            .map(|v| Milestone::unpack_unchecked(&mut v.as_ref()).unwrap()))
    }
}

#[async_trait::async_trait]
impl Fetch<(), SnapshotInfo> for Storage {
    async fn fetch(&self, (): &()) -> Result<Option<SnapshotInfo>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_SNAPSHOT_INFO)?
            .get([0x00u8])?
            // Unpacking from storage is fine.
            .map(|v| SnapshotInfo::unpack_unchecked(&mut v.as_ref()).unwrap()))
    }
}

#[async_trait::async_trait]
impl Fetch<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn fetch(&self, sep: &SolidEntryPoint) -> Result<Option<MilestoneIndex>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?
            .get(sep.as_ref())?
            // Unpacking from storage is fine.
            .map(|v| MilestoneIndex::unpack_unchecked(&mut v.as_ref()).unwrap()))
    }
}

#[async_trait::async_trait]
impl Fetch<MilestoneIndex, OutputDiff> for Storage {
    async fn fetch(&self, index: &MilestoneIndex) -> Result<Option<OutputDiff>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF)?
            .get(index.pack_new())?
            // Unpacking from storage is fine.
            .map(|v| OutputDiff::unpack_unchecked(&mut v.as_ref()).unwrap()))
    }
}

#[async_trait::async_trait]
impl Fetch<Address, Balance> for Storage {
    async fn fetch(&self, address: &Address) -> Result<Option<Balance>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_ADDRESS_TO_BALANCE)?
            .get(address.pack_new())?
            // Unpacking from storage is fine.
            .map(|v| Balance::unpack_unchecked(&mut v.as_ref()).unwrap()))
    }
}

#[async_trait::async_trait]
impl Fetch<MilestoneIndex, Vec<UnreferencedMessage>> for Storage {
    async fn fetch(
        &self,
        index: &MilestoneIndex,
    ) -> Result<Option<Vec<UnreferencedMessage>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .open_tree(TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?
                .scan_prefix(index.pack_new())
                .map(|result| {
                    let (key, _) = result?;
                    let (_, unreferenced_message) = key.split_at(std::mem::size_of::<MilestoneIndex>());
                    // Unpacking from storage is fine.
                    let unreferenced_message: [u8; MESSAGE_ID_LENGTH] = unreferenced_message.try_into().unwrap();
                    Ok(UnreferencedMessage::from(MessageId::from(unreferenced_message)))
                })
                .collect::<Result<Vec<UnreferencedMessage>, Self::Error>>()?,
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<MilestoneIndex, Vec<Receipt>> for Storage {
    async fn fetch(&self, index: &MilestoneIndex) -> Result<Option<Vec<Receipt>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .open_tree(TREE_MILESTONE_INDEX_TO_RECEIPT)?
                .scan_prefix(index.pack_new())
                .map(|result| {
                    let (mut key, _) = result?;
                    let (_, receipt) = key.split_at_mut(std::mem::size_of::<MilestoneIndex>());
                    // Unpacking from storage is fine.
                    #[allow(clippy::useless_asref)]
                    Ok(Receipt::unpack_unchecked(&mut receipt.as_ref()).unwrap())
                })
                .collect::<Result<Vec<Receipt>, Self::Error>>()?,
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<bool, Vec<TreasuryOutput>> for Storage {
    async fn fetch(&self, spent: &bool) -> Result<Option<Vec<TreasuryOutput>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .open_tree(TREE_SPENT_TO_TREASURY_OUTPUT)?
                .scan_prefix(spent.pack_new())
                .map(|result| {
                    let (mut key, _) = result?;
                    let (_, output) = key.split_at_mut(std::mem::size_of::<bool>());
                    // Unpacking from storage is fine.
                    #[allow(clippy::useless_asref)]
                    Ok(TreasuryOutput::unpack_unchecked(&mut output.as_ref()).unwrap())
                })
                .collect::<Result<Vec<TreasuryOutput>, Self::Error>>()?,
        ))
    }
}
