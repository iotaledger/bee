// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage::*};

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
use bee_storage::access::Delete;
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unconfirmed_message::UnconfirmedMessage,
};

fn delete<T: AsRef<[u8]>>(
    storage: &Storage,
    cf_str: &'static str,
    key: T,
) -> Result<(), <Storage as StorageBackend>::Error> {
    let cf = storage
        .inner
        .cf_handle(cf_str)
        .ok_or(Error::UnknownColumnFamily(cf_str))?;

    storage.inner.delete_cf(cf, key)?;

    Ok(())
}

#[async_trait::async_trait]
impl Delete<MessageId, Message> for Storage {
    async fn delete(&self, message_id: &MessageId) -> Result<(), <Self as StorageBackend>::Error> {
        delete(self, CF_MESSAGE_ID_TO_MESSAGE, message_id)
    }
}

#[async_trait::async_trait]
impl Delete<MessageId, MessageMetadata> for Storage {
    async fn delete(&self, message_id: &MessageId) -> Result<(), <Self as StorageBackend>::Error> {
        delete(self, CF_MESSAGE_ID_TO_METADATA, message_id)
    }
}

#[async_trait::async_trait]
impl Delete<(MessageId, MessageId), ()> for Storage {
    async fn delete(&self, (parent, child): &(MessageId, MessageId)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        delete(self, CF_MESSAGE_ID_TO_MESSAGE_ID, key)
    }
}

#[async_trait::async_trait]
impl Delete<(PaddedIndex, MessageId), ()> for Storage {
    async fn delete(
        &self,
        (index, message_id): &(PaddedIndex, MessageId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        delete(self, CF_INDEX_TO_MESSAGE_ID, key)
    }
}

#[async_trait::async_trait]
impl Delete<OutputId, CreatedOutput> for Storage {
    async fn delete(&self, output_id: &OutputId) -> Result<(), <Self as StorageBackend>::Error> {
        delete(self, CF_OUTPUT_ID_TO_CREATED_OUTPUT, output_id.pack_new())
    }
}

#[async_trait::async_trait]
impl Delete<OutputId, ConsumedOutput> for Storage {
    async fn delete(&self, output_id: &OutputId) -> Result<(), <Self as StorageBackend>::Error> {
        delete(self, CF_OUTPUT_ID_TO_CONSUMED_OUTPUT, output_id.pack_new())
    }
}

#[async_trait::async_trait]
impl Delete<Unspent, ()> for Storage {
    async fn delete(&self, unspent: &Unspent) -> Result<(), <Self as StorageBackend>::Error> {
        delete(self, CF_OUTPUT_ID_UNSPENT, unspent.pack_new())
    }
}

#[async_trait::async_trait]
impl Delete<(Ed25519Address, OutputId), ()> for Storage {
    async fn delete(
        &self,
        (address, output_id): &(Ed25519Address, OutputId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        delete(self, CF_ED25519_ADDRESS_TO_OUTPUT_ID, key)
    }
}

#[async_trait::async_trait]
impl Delete<(), LedgerIndex> for Storage {
    async fn delete(&self, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        delete(self, CF_LEDGER_INDEX, [0x00u8])
    }
}

#[async_trait::async_trait]
impl Delete<MilestoneIndex, Milestone> for Storage {
    async fn delete(&self, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        delete(self, CF_MILESTONE_INDEX_TO_MILESTONE, index.pack_new())
    }
}

#[async_trait::async_trait]
impl Delete<(), SnapshotInfo> for Storage {
    async fn delete(&self, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        delete(self, CF_SNAPSHOT_INFO, [0x00u8])
    }
}

#[async_trait::async_trait]
impl Delete<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn delete(&self, sep: &SolidEntryPoint) -> Result<(), <Self as StorageBackend>::Error> {
        // TODO SEP AS REF
        delete(self, CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX, sep.pack_new())
    }
}

#[async_trait::async_trait]
impl Delete<MilestoneIndex, OutputDiff> for Storage {
    async fn delete(&self, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        delete(self, CF_MILESTONE_INDEX_TO_OUTPUT_DIFF, index.pack_new())
    }
}

#[async_trait::async_trait]
impl Delete<Address, Balance> for Storage {
    async fn delete(&self, address: &Address) -> Result<(), <Self as StorageBackend>::Error> {
        // TODO as ref ?
        delete(self, CF_ADDRESS_TO_BALANCE, address.pack_new())
    }
}

#[async_trait::async_trait]
impl Delete<(MilestoneIndex, UnconfirmedMessage), ()> for Storage {
    async fn delete(
        &self,
        (index, unconfirmed_message): &(MilestoneIndex, UnconfirmedMessage),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(unconfirmed_message.as_ref());

        delete(self, CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE, key)
    }
}

#[async_trait::async_trait]
impl Delete<(MilestoneIndex, Receipt), ()> for Storage {
    async fn delete(
        &self,
        (index, receipt): &(MilestoneIndex, Receipt),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(&receipt.pack_new());

        delete(self, CF_MILESTONE_INDEX_TO_RECEIPT, key)
    }
}

#[async_trait::async_trait]
impl Delete<(bool, TreasuryOutput), ()> for Storage {
    async fn delete(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = spent.pack_new();
        key.extend_from_slice(&output.pack_new());

        delete(self, CF_SPENT_TO_TREASURY_OUTPUT, key)
    }
}
