// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Exist access operations.

use bee_ledger::types::{
    snapshot::info::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput,
    Unspent,
};
use bee_message::{
    address::Ed25519Address, milestone::Milestone, output::OutputId, payload::milestone::MilestoneIndex, Message,
    MessageId,
};
use bee_storage::{access::Exist, backend::StorageBackend};
use bee_tangle::{
    message_metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};
use packable::PackableExt;

use crate::{storage::Storage, trees::*};

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

impl Exist<OutputId, CreatedOutput> for Storage {
    fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_TO_CREATED_OUTPUT)?
            .contains_key(output_id.pack_to_vec())?)
    }
}

impl Exist<OutputId, ConsumedOutput> for Storage {
    fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT)?
            .contains_key(output_id.pack_to_vec())?)
    }
}

impl Exist<Unspent, ()> for Storage {
    fn exist(&self, unspent: &Unspent) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_UNSPENT)?
            .contains_key(unspent.pack_to_vec())?)
    }
}

impl Exist<(Ed25519Address, OutputId), ()> for Storage {
    fn exist(
        &self,
        (address, output_id): &(Ed25519Address, OutputId),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_to_vec());

        Ok(self
            .inner
            .open_tree(TREE_ED25519_ADDRESS_TO_OUTPUT_ID)?
            .contains_key(key)?)
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
            .contains_key(index.pack_to_vec())?)
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
            .contains_key(sep.pack_to_vec())?)
    }
}

impl Exist<MilestoneIndex, OutputDiff> for Storage {
    fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF)?
            .contains_key(index.pack_to_vec())?)
    }
}

impl Exist<(MilestoneIndex, UnreferencedMessage), ()> for Storage {
    fn exist(
        &self,
        (index, unreferenced_message): &(MilestoneIndex, UnreferencedMessage),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.pack_to_vec();
        key.extend_from_slice(unreferenced_message.as_ref());

        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?
            .contains_key(key)?)
    }
}

impl Exist<(MilestoneIndex, Receipt), ()> for Storage {
    fn exist(&self, (index, receipt): &(MilestoneIndex, Receipt)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.pack_to_vec();
        key.extend_from_slice(&receipt.pack_to_vec());

        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_RECEIPT)?
            .contains_key(key)?)
    }
}

impl Exist<(bool, TreasuryOutput), ()> for Storage {
    fn exist(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = spent.pack_to_vec();
        key.extend_from_slice(&output.pack_to_vec());

        Ok(self.inner.open_tree(TREE_SPENT_TO_TREASURY_OUTPUT)?.contains_key(key)?)
    }
}
