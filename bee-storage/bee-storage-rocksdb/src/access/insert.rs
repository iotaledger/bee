// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

use bee_ledger::types::{
    snapshot::info::SnapshotInfo, Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt,
    TreasuryOutput, Unspent,
};
use bee_message::{
    address::{Address, Ed25519Address},
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::indexation::PaddedIndex,
    Message, MessageId,
};
use bee_packable::PackableExt;
use bee_storage::{access::Insert, system::System};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

impl Insert<u8, System> for Storage {
    fn insert(&self, key: &u8, value: &System) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .put_cf(self.cf_handle(CF_SYSTEM)?, [*key], value.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<MessageId, Message> for Storage {
    fn insert(&self, message_id: &MessageId, message: &Message) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?,
            message_id,
            message.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<MessageId, MessageMetadata> for Storage {
    fn insert(
        &self,
        message_id: &MessageId,
        metadata: &MessageMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_MESSAGE_ID_TO_METADATA)?,
            message_id,
            metadata.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<(MessageId, MessageId), ()> for Storage {
    fn insert(&self, (parent, child): &(MessageId, MessageId), (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        self.inner
            .put_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)?, key, [])?;

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

        self.inner.put_cf(self.cf_handle(CF_INDEX_TO_MESSAGE_ID)?, key, [])?;

        Ok(())
    }
}

impl Insert<OutputId, CreatedOutput> for Storage {
    fn insert(&self, output_id: &OutputId, output: &CreatedOutput) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)?,
            output_id.pack_to_vec(),
            output.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<OutputId, ConsumedOutput> for Storage {
    fn insert(&self, output_id: &OutputId, output: &ConsumedOutput) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)?,
            output_id.pack_to_vec(),
            output.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<Unspent, ()> for Storage {
    fn insert(&self, unspent: &Unspent, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .put_cf(self.cf_handle(CF_OUTPUT_ID_UNSPENT)?, unspent.pack_to_vec(), [])?;

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
        key.extend_from_slice(&output_id.pack_to_vec());

        self.inner
            .put_cf(self.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)?, key, [])?;

        Ok(())
    }
}

impl Insert<(), LedgerIndex> for Storage {
    fn insert(&self, (): &(), index: &LedgerIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .put_cf(self.cf_handle(CF_LEDGER_INDEX)?, [0x00u8], index.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<MilestoneIndex, Milestone> for Storage {
    fn insert(&self, index: &MilestoneIndex, milestone: &Milestone) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE)?,
            index.pack_to_vec(),
            milestone.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<(), SnapshotInfo> for Storage {
    fn insert(&self, (): &(), info: &SnapshotInfo) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .put_cf(self.cf_handle(CF_SNAPSHOT_INFO)?, [0x00u8], info.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<SolidEntryPoint, MilestoneIndex> for Storage {
    fn insert(&self, sep: &SolidEntryPoint, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?,
            sep.as_ref(),
            index.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<MilestoneIndex, OutputDiff> for Storage {
    fn insert(&self, index: &MilestoneIndex, diff: &OutputDiff) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)?,
            index.pack_to_vec(),
            diff.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<Address, Balance> for Storage {
    fn insert(&self, address: &Address, balance: &Balance) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_ADDRESS_TO_BALANCE)?,
            address.pack_to_vec(),
            balance.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<(MilestoneIndex, UnreferencedMessage), ()> for Storage {
    fn insert(
        &self,
        (index, unreferenced_message): &(MilestoneIndex, UnreferencedMessage),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_to_vec();
        key.extend_from_slice(unreferenced_message.as_ref());

        self.inner
            .put_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?, key, [])?;

        Ok(())
    }
}

impl Insert<(MilestoneIndex, Receipt), ()> for Storage {
    fn insert(
        &self,
        (index, receipt): &(MilestoneIndex, Receipt),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_to_vec();
        key.extend_from_slice(&receipt.pack_to_vec());

        self.inner
            .put_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)?, key, [])?;

        Ok(())
    }
}

impl Insert<(bool, TreasuryOutput), ()> for Storage {
    fn insert(&self, (spent, output): &(bool, TreasuryOutput), (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = spent.pack_to_vec();
        key.extend_from_slice(&output.pack_to_vec());

        self.inner
            .put_cf(self.cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)?, key, [])?;

        Ok(())
    }
}
