// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::*;

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
use bee_storage::access::Truncate;
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unconfirmed_message::UnconfirmedMessage,
};

fn truncate(storage: &Storage, cf_str: &'static str) -> Result<(), <Storage as StorageBackend>::Error> {
    let cf_handle = storage.cf_handle(cf_str)?;

    let mut iter = storage.inner.raw_iterator_cf(cf_handle);

    // Seek to the first key.
    iter.seek_to_first();
    // Grab the first key if it exists.
    let first = if let Some(first) = iter.key() {
        first.to_vec()
    } else {
        // There are no keys to remove.
        return Ok(());
    };

    iter.seek_to_last();
    // Grab the last key if it exists.
    let last = if let Some(last) = iter.key() {
        let mut last = last.to_vec();
        // `delete_range_cf` excludes the last key in the range so a byte is added to be sure the last key is included.
        last.push(u8::MAX);
        last
    } else {
        // There are no keys to remove.
        return Ok(());
    };

    storage.inner.delete_range_cf(cf_handle, first, last)?;

    Ok(())
}

#[async_trait::async_trait]
impl Truncate<MessageId, Message> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_MESSAGE_ID_TO_MESSAGE)
    }
}

#[async_trait::async_trait]
impl Truncate<MessageId, MessageMetadata> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_MESSAGE_ID_TO_METADATA)
    }
}

#[async_trait::async_trait]
impl Truncate<(MessageId, MessageId), ()> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_MESSAGE_ID_TO_MESSAGE_ID)
    }
}

#[async_trait::async_trait]
impl Truncate<(PaddedIndex, MessageId), ()> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_INDEX_TO_MESSAGE_ID)
    }
}

#[async_trait::async_trait]
impl Truncate<OutputId, CreatedOutput> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_OUTPUT_ID_TO_CREATED_OUTPUT)
    }
}

#[async_trait::async_trait]
impl Truncate<OutputId, ConsumedOutput> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)
    }
}

#[async_trait::async_trait]
impl Truncate<Unspent, ()> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_OUTPUT_ID_UNSPENT)
    }
}

#[async_trait::async_trait]
impl Truncate<(Ed25519Address, OutputId), ()> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_ED25519_ADDRESS_TO_OUTPUT_ID)
    }
}

#[async_trait::async_trait]
impl Truncate<(), LedgerIndex> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_LEDGER_INDEX)
    }
}

#[async_trait::async_trait]
impl Truncate<MilestoneIndex, Milestone> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_MILESTONE_INDEX_TO_MILESTONE)
    }
}

#[async_trait::async_trait]
impl Truncate<(), SnapshotInfo> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_SNAPSHOT_INFO)
    }
}

#[async_trait::async_trait]
impl Truncate<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)
    }
}

#[async_trait::async_trait]
impl Truncate<MilestoneIndex, OutputDiff> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)
    }
}

#[async_trait::async_trait]
impl Truncate<Address, Balance> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_ADDRESS_TO_BALANCE)
    }
}

#[async_trait::async_trait]
impl Truncate<(MilestoneIndex, UnconfirmedMessage), ()> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE)
    }
}

#[async_trait::async_trait]
impl Truncate<(MilestoneIndex, Receipt), ()> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_MILESTONE_INDEX_TO_RECEIPT)
    }
}

#[async_trait::async_trait]
impl Truncate<(bool, TreasuryOutput), ()> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        truncate(self, CF_SPENT_TO_TREASURY_OUTPUT)
    }
}
