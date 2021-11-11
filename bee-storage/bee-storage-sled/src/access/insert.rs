// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Insert access operations.

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
use bee_storage::{access::Insert, backend::StorageBackend, system::System};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

impl Insert<u8, System> for Storage {
    fn insert(&self, key: &u8, value: &System) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.insert(&[*key], value.pack_new())?;

        Ok(())
    }
}

impl Insert<MessageId, Message> for Storage {
    fn insert(&self, message_id: &MessageId, message: &Message) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_MESSAGE_ID_TO_MESSAGE)?
            .insert(message_id, message.pack_new())?;

        Ok(())
    }
}

impl Insert<MessageId, MessageMetadata> for Storage {
    fn insert(
        &self,
        message_id: &MessageId,
        metadata: &MessageMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_MESSAGE_ID_TO_METADATA)?
            .insert(message_id, metadata.pack_new())?;

        Ok(())
    }
}

impl Insert<(MessageId, MessageId), ()> for Storage {
    fn insert(&self, (parent, child): &(MessageId, MessageId), (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        self.inner.open_tree(TREE_MESSAGE_ID_TO_MESSAGE_ID)?.insert(key, &[])?;

        Ok(())
    }
}

impl Insert<(PaddedIndex, MessageId), ()> for Storage {
    fn insert(
        &self,
        (index, message_id): &(PaddedIndex, MessageId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        self.inner.open_tree(TREE_INDEX_TO_MESSAGE_ID)?.insert(key, &[])?;

        Ok(())
    }
}

impl Insert<OutputId, CreatedOutput> for Storage {
    fn insert(&self, output_id: &OutputId, output: &CreatedOutput) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_OUTPUT_ID_TO_CREATED_OUTPUT)?
            .insert(output_id.pack_new(), output.pack_new())?;

        Ok(())
    }
}

impl Insert<OutputId, ConsumedOutput> for Storage {
    fn insert(&self, output_id: &OutputId, output: &ConsumedOutput) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT)?
            .insert(output_id.pack_new(), output.pack_new())?;

        Ok(())
    }
}

impl Insert<Unspent, ()> for Storage {
    fn insert(&self, unspent: &Unspent, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_OUTPUT_ID_UNSPENT)?
            .insert(unspent.pack_new(), &[])?;

        Ok(())
    }
}

impl Insert<(Ed25519Address, OutputId), ()> for Storage {
    fn insert(
        &self,
        (address, output_id): &(Ed25519Address, OutputId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        self.inner
            .open_tree(TREE_ED25519_ADDRESS_TO_OUTPUT_ID)?
            .insert(key, &[])?;

        Ok(())
    }
}

impl Insert<(AliasAddress, OutputId), ()> for Storage {
    fn insert(
        &self,
        (address, output_id): &(AliasAddress, OutputId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        self.inner
            .open_tree(TREE_ALIAS_ADDRESS_TO_OUTPUT_ID)?
            .insert(key, &[])?;

        Ok(())
    }
}

impl Insert<(NftAddress, OutputId), ()> for Storage {
    fn insert(
        &self,
        (address, output_id): &(NftAddress, OutputId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        self.inner.open_tree(TREE_NFT_ADDRESS_TO_OUTPUT_ID)?.insert(key, &[])?;

        Ok(())
    }
}

impl Insert<(), LedgerIndex> for Storage {
    fn insert(&self, (): &(), index: &LedgerIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_LEDGER_INDEX)?
            .insert([0x00u8], index.pack_new())?;

        Ok(())
    }
}

impl Insert<MilestoneIndex, Milestone> for Storage {
    fn insert(&self, index: &MilestoneIndex, milestone: &Milestone) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_MILESTONE_INDEX_TO_MILESTONE)?
            .insert(index.pack_new(), milestone.pack_new())?;

        Ok(())
    }
}

impl Insert<(), SnapshotInfo> for Storage {
    fn insert(&self, (): &(), info: &SnapshotInfo) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_SNAPSHOT_INFO)?
            .insert([0x00u8], info.pack_new())?;

        Ok(())
    }
}

impl Insert<SolidEntryPoint, MilestoneIndex> for Storage {
    fn insert(&self, sep: &SolidEntryPoint, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?
            .insert(sep.as_ref(), index.pack_new())?;

        Ok(())
    }
}

impl Insert<MilestoneIndex, OutputDiff> for Storage {
    fn insert(&self, index: &MilestoneIndex, diff: &OutputDiff) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF)?
            .insert(index.pack_new(), diff.pack_new())?;

        Ok(())
    }
}

impl Insert<Address, Balance> for Storage {
    fn insert(&self, address: &Address, balance: &Balance) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_ADDRESS_TO_BALANCE)?
            .insert(address.pack_new(), balance.pack_new())?;

        Ok(())
    }
}

impl Insert<(MilestoneIndex, UnreferencedMessage), ()> for Storage {
    fn insert(
        &self,
        (index, unreferenced_message): &(MilestoneIndex, UnreferencedMessage),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(unreferenced_message.as_ref());

        self.inner
            .open_tree(TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?
            .insert(key, &[])?;

        Ok(())
    }
}

impl Insert<(MilestoneIndex, Receipt), ()> for Storage {
    fn insert(
        &self,
        (index, receipt): &(MilestoneIndex, Receipt),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(&receipt.pack_new());

        self.inner
            .open_tree(TREE_MILESTONE_INDEX_TO_RECEIPT)?
            .insert(key, &[])?;

        Ok(())
    }
}

impl Insert<(bool, TreasuryOutput), ()> for Storage {
    fn insert(&self, (spent, output): &(bool, TreasuryOutput), (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = spent.pack_new();
        key.extend_from_slice(&output.pack_new());

        self.inner.open_tree(TREE_SPENT_TO_TREASURY_OUTPUT)?.insert(key, &[])?;

        Ok(())
    }
}
