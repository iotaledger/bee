// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Exist access operations.

use crate::{storage::Storage, trees::*};

use bee_common::packable::Packable;
use bee_ledger::types::{
    snapshot::info::SnapshotInfo, Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt,
    TreasuryOutput, Unspent,
};
use bee_message::{
    address::{Address, AliasAddress, Ed25519Address, NftAddress},
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::indexation::PaddedIndex,
    Message, MessageId,
};
use bee_storage::{access::Exist, backend::StorageBackend};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

impl Exist<MessageId, Message> for Storage {
    fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MESSAGE_ID_TO_MESSAGE)?
            .contains_key(message_id)?)
    }
}

impl Exist<MessageId, MessageMetadata> for Storage {
    fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MESSAGE_ID_TO_METADATA)?
            .contains_key(message_id)?)
    }
}

impl Exist<(MessageId, MessageId), ()> for Storage {
    fn exist(&self, (parent, child): &(MessageId, MessageId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        Ok(self.inner.open_tree(TREE_MESSAGE_ID_TO_MESSAGE_ID)?.contains_key(key)?)
    }
}

impl Exist<(PaddedIndex, MessageId), ()> for Storage {
    fn exist(&self, (index, message_id): &(PaddedIndex, MessageId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        Ok(self.inner.open_tree(TREE_INDEX_TO_MESSAGE_ID)?.contains_key(key)?)
    }
}

impl Exist<OutputId, CreatedOutput> for Storage {
    fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_TO_CREATED_OUTPUT)?
            .contains_key(output_id.pack_new())?)
    }
}

impl Exist<OutputId, ConsumedOutput> for Storage {
    fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT)?
            .contains_key(output_id.pack_new())?)
    }
}

impl Exist<Unspent, ()> for Storage {
    fn exist(&self, unspent: &Unspent) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_UNSPENT)?
            .contains_key(unspent.pack_new())?)
    }
}

impl Exist<(Ed25519Address, OutputId), ()> for Storage {
    fn exist(
        &self,
        (address, output_id): &(Ed25519Address, OutputId),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        Ok(self
            .inner
            .open_tree(TREE_ED25519_ADDRESS_TO_OUTPUT_ID)?
            .contains_key(key)?)
    }
}

impl Exist<(AliasAddress, OutputId), ()> for Storage {
    fn exist(&self, (address, output_id): &(AliasAddress, OutputId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        Ok(self
            .inner
            .open_tree(TREE_ALIAS_ADDRESS_TO_OUTPUT_ID)?
            .contains_key(key)?)
    }
}

impl Exist<(NftAddress, OutputId), ()> for Storage {
    fn exist(&self, (address, output_id): &(NftAddress, OutputId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        Ok(self.inner.open_tree(TREE_NFT_ADDRESS_TO_OUTPUT_ID)?.contains_key(key)?)
    }
}

impl Exist<(), LedgerIndex> for Storage {
    fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self.inner.open_tree(TREE_LEDGER_INDEX)?.contains_key([0x00u8])?)
    }
}

impl Exist<MilestoneIndex, Milestone> for Storage {
    fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_MILESTONE)?
            .contains_key(index.pack_new())?)
    }
}

impl Exist<(), SnapshotInfo> for Storage {
    fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self.inner.open_tree(TREE_SNAPSHOT_INFO)?.contains_key([0x00u8])?)
    }
}

impl Exist<SolidEntryPoint, MilestoneIndex> for Storage {
    fn exist(&self, sep: &SolidEntryPoint) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?
            .contains_key(sep.pack_new())?)
    }
}

impl Exist<MilestoneIndex, OutputDiff> for Storage {
    fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF)?
            .contains_key(index.pack_new())?)
    }
}

impl Exist<Address, Balance> for Storage {
    fn exist(&self, address: &Address) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_ADDRESS_TO_BALANCE)?
            .contains_key(address.pack_new())?)
    }
}

impl Exist<(MilestoneIndex, UnreferencedMessage), ()> for Storage {
    fn exist(
        &self,
        (index, unreferenced_message): &(MilestoneIndex, UnreferencedMessage),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(unreferenced_message.as_ref());

        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?
            .contains_key(key)?)
    }
}

impl Exist<(MilestoneIndex, Receipt), ()> for Storage {
    fn exist(&self, (index, receipt): &(MilestoneIndex, Receipt)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(&receipt.pack_new());

        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_RECEIPT)?
            .contains_key(key)?)
    }
}

impl Exist<(bool, TreasuryOutput), ()> for Storage {
    fn exist(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = spent.pack_new();
        key.extend_from_slice(&output.pack_new());

        Ok(self.inner.open_tree(TREE_SPENT_TO_TREASURY_OUTPUT)?.contains_key(key)?)
    }
}
