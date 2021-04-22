// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{storage::*, system::System};

use bee_common::packable::Packable;
use bee_ledger::{
    snapshot::info::SnapshotInfo,
    types::{Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent},
};
use bee_message::{
    address::{Address, Ed25519Address},
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::indexation::PaddedIndex,
    Message, MessageId,
};
use bee_storage::access::Insert;
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unconfirmed_message::UnconfirmedMessage,
};

fn insert<K: AsRef<[u8]>, V: AsRef<[u8]>>(
    storage: &Storage,
    cf_str: &'static str,
    key: K,
    value: V,
) -> Result<(), <Storage as StorageBackend>::Error> {
    let cf = storage.cf_handle(cf_str)?;

    storage.inner.put_cf(cf, key, value)?;

    Ok(())
}

#[async_trait::async_trait]
impl Insert<u8, System> for Storage {
    async fn insert(&self, key: &u8, value: &System) -> Result<(), <Self as StorageBackend>::Error> {
        insert(self, CF_SYSTEM, [*key], value.pack_new())
    }
}

#[async_trait::async_trait]
impl Insert<MessageId, Message> for Storage {
    async fn insert(&self, message_id: &MessageId, message: &Message) -> Result<(), <Self as StorageBackend>::Error> {
        insert(self, CF_MESSAGE_ID_TO_MESSAGE, message_id, message.pack_new())
    }
}

#[async_trait::async_trait]
impl Insert<MessageId, MessageMetadata> for Storage {
    async fn insert(
        &self,
        message_id: &MessageId,
        metadata: &MessageMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        insert(self, CF_MESSAGE_ID_TO_METADATA, message_id, metadata.pack_new())
    }
}

#[async_trait::async_trait]
impl Insert<(MessageId, MessageId), ()> for Storage {
    async fn insert(
        &self,
        (parent, child): &(MessageId, MessageId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        insert(self, CF_MESSAGE_ID_TO_MESSAGE_ID, key, [])
    }
}

#[async_trait::async_trait]
impl Insert<(PaddedIndex, MessageId), ()> for Storage {
    async fn insert(
        &self,
        (index, message_id): &(PaddedIndex, MessageId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        insert(self, CF_INDEX_TO_MESSAGE_ID, key, [])
    }
}

#[async_trait::async_trait]
impl Insert<OutputId, CreatedOutput> for Storage {
    async fn insert(
        &self,
        output_id: &OutputId,
        output: &CreatedOutput,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        insert(
            self,
            CF_OUTPUT_ID_TO_CREATED_OUTPUT,
            output_id.pack_new(),
            output.pack_new(),
        )
    }
}

#[async_trait::async_trait]
impl Insert<OutputId, ConsumedOutput> for Storage {
    async fn insert(
        &self,
        output_id: &OutputId,
        output: &ConsumedOutput,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        insert(
            self,
            CF_OUTPUT_ID_TO_CONSUMED_OUTPUT,
            output_id.pack_new(),
            output.pack_new(),
        )
    }
}

#[async_trait::async_trait]
impl Insert<Unspent, ()> for Storage {
    async fn insert(&self, unspent: &Unspent, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        insert(self, CF_OUTPUT_ID_UNSPENT, unspent.pack_new(), [])
    }
}

#[async_trait::async_trait]
impl Insert<(Ed25519Address, OutputId), ()> for Storage {
    async fn insert(
        &self,
        (address, output_id): &(Ed25519Address, OutputId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        insert(self, CF_ED25519_ADDRESS_TO_OUTPUT_ID, key, [])
    }
}

#[async_trait::async_trait]
impl Insert<(), LedgerIndex> for Storage {
    async fn insert(&self, (): &(), index: &LedgerIndex) -> Result<(), <Self as StorageBackend>::Error> {
        insert(self, CF_LEDGER_INDEX, [0x00u8], index.pack_new())
    }
}

#[async_trait::async_trait]
impl Insert<MilestoneIndex, Milestone> for Storage {
    async fn insert(
        &self,
        index: &MilestoneIndex,
        milestone: &Milestone,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        insert(
            self,
            CF_MILESTONE_INDEX_TO_MILESTONE,
            index.pack_new(),
            milestone.pack_new(),
        )
    }
}

#[async_trait::async_trait]
impl Insert<(), SnapshotInfo> for Storage {
    async fn insert(&self, (): &(), info: &SnapshotInfo) -> Result<(), <Self as StorageBackend>::Error> {
        insert(self, CF_SNAPSHOT_INFO, [0x00u8], info.pack_new())
    }
}

#[async_trait::async_trait]
impl Insert<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn insert(
        &self,
        sep: &SolidEntryPoint,
        index: &MilestoneIndex,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        // TODO SEP AS REF
        insert(
            self,
            CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX,
            sep.pack_new(),
            index.pack_new(),
        )
    }
}

#[async_trait::async_trait]
impl Insert<MilestoneIndex, OutputDiff> for Storage {
    async fn insert(&self, index: &MilestoneIndex, diff: &OutputDiff) -> Result<(), <Self as StorageBackend>::Error> {
        insert(
            self,
            CF_MILESTONE_INDEX_TO_OUTPUT_DIFF,
            index.pack_new(),
            diff.pack_new(),
        )
    }
}

#[async_trait::async_trait]
impl Insert<Address, Balance> for Storage {
    async fn insert(&self, address: &Address, balance: &Balance) -> Result<(), <Self as StorageBackend>::Error> {
        insert(self, CF_ADDRESS_TO_BALANCE, address.pack_new(), balance.pack_new())
    }
}

#[async_trait::async_trait]
impl Insert<(MilestoneIndex, UnconfirmedMessage), ()> for Storage {
    async fn insert(
        &self,
        (index, unconfirmed_message): &(MilestoneIndex, UnconfirmedMessage),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(unconfirmed_message.as_ref());

        insert(self, CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE, key, [])
    }
}

#[async_trait::async_trait]
impl Insert<(MilestoneIndex, Receipt), ()> for Storage {
    async fn insert(
        &self,
        (index, receipt): &(MilestoneIndex, Receipt),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(&receipt.pack_new());

        insert(self, CF_MILESTONE_INDEX_TO_RECEIPT, key, [])
    }
}

#[async_trait::async_trait]
impl Insert<(bool, TreasuryOutput), ()> for Storage {
    async fn insert(
        &self,
        (spent, output): &(bool, TreasuryOutput),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = spent.pack_new();
        key.extend_from_slice(&output.pack_new());

        insert(self, CF_SPENT_TO_TREASURY_OUTPUT, key, [])
    }
}
