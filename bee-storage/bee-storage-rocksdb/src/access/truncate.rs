// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage::*};

use bee_ledger::model::{Balance, OutputDiff, Unspent};
use bee_message::{
    ledger_index::LedgerIndex,
    milestone::{Milestone, MilestoneIndex},
    payload::{
        indexation::{HashedIndex, HASHED_INDEX_LENGTH},
        transaction::{
            Address, ConsumedOutput, CreatedOutput, Ed25519Address, OutputId, ED25519_ADDRESS_LENGTH, OUTPUT_ID_LENGTH,
        },
    },
    solid_entry_point::SolidEntryPoint,
    Message, MessageId, MESSAGE_ID_LENGTH,
};
use bee_snapshot::info::SnapshotInfo;
use bee_storage::access::Truncate;
use bee_tangle::metadata::MessageMetadata;

#[async_trait::async_trait]
impl Truncate<MessageId, Message> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE))?;

        self.inner
            .delete_range_cf(cf, [0x00u8; MESSAGE_ID_LENGTH], [0xffu8; MESSAGE_ID_LENGTH])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<MessageId, MessageMetadata> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_METADATA)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_METADATA))?;

        self.inner
            .delete_range_cf(cf, [0x00u8; MESSAGE_ID_LENGTH], [0xffu8; MESSAGE_ID_LENGTH])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<(MessageId, MessageId), ()> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE_ID))?;

        self.inner
            .delete_range_cf(cf, [0x00u8; 2 * MESSAGE_ID_LENGTH], [0xffu8; 2 * MESSAGE_ID_LENGTH])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<(HashedIndex, MessageId), ()> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_INDEX_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_INDEX_TO_MESSAGE_ID))?;

        self.inner.delete_range_cf(
            cf,
            [0x00u8; HASHED_INDEX_LENGTH + MESSAGE_ID_LENGTH],
            [0xffu8; HASHED_INDEX_LENGTH + MESSAGE_ID_LENGTH],
        )?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<OutputId, CreatedOutput> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CREATED_OUTPUT))?;

        self.inner
            .delete_range_cf(cf, [0x00u8; OUTPUT_ID_LENGTH], [0xffu8; OUTPUT_ID_LENGTH])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<OutputId, ConsumedOutput> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT))?;

        self.inner
            .delete_range_cf(cf, [0x00u8; OUTPUT_ID_LENGTH], [0xffu8; OUTPUT_ID_LENGTH])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<Unspent, ()> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_UNSPENT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_UNSPENT))?;

        self.inner
            .delete_range_cf(cf, [0x00u8; OUTPUT_ID_LENGTH], [0xffu8; OUTPUT_ID_LENGTH])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<(Ed25519Address, OutputId), ()> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)
            .ok_or(Error::UnknownCf(CF_ED25519_ADDRESS_TO_OUTPUT_ID))?;

        self.inner.delete_range_cf(
            cf,
            [0x00u8; ED25519_ADDRESS_LENGTH + OUTPUT_ID_LENGTH],
            [0xffu8; ED25519_ADDRESS_LENGTH + OUTPUT_ID_LENGTH],
        )?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<(), LedgerIndex> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_LEDGER_INDEX)
            .ok_or(Error::UnknownCf(CF_LEDGER_INDEX))?;

        self.inner.delete_range_cf(cf, [0x00u8], [0xffu8])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<MilestoneIndex, Milestone> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_MILESTONE))?;

        self.inner.delete_range_cf(
            cf,
            [0x00u8; std::mem::size_of::<MilestoneIndex>()],
            [0xffu8; std::mem::size_of::<MilestoneIndex>()],
        )?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<(), SnapshotInfo> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SNAPSHOT_INFO)
            .ok_or(Error::UnknownCf(CF_SNAPSHOT_INFO))?;

        self.inner.delete_range_cf(cf, [0x00u8], [0xffu8])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)
            .ok_or(Error::UnknownCf(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX))?;

        self.inner
            .delete_range_cf(cf, [0x00u8; MESSAGE_ID_LENGTH], [0xffu8; MESSAGE_ID_LENGTH])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<MilestoneIndex, OutputDiff> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF))?;

        self.inner.delete_range_cf(
            cf,
            [0x00u8; std::mem::size_of::<MilestoneIndex>()],
            [0xffu8; std::mem::size_of::<MilestoneIndex>()],
        )?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Truncate<Address, Balance> for Storage {
    async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ADDRESS_TO_BALANCE)
            .ok_or(Error::UnknownCf(CF_ADDRESS_TO_BALANCE))?;

        // TODO check that this is fine
        self.inner.delete_range_cf(cf, [0x00u8; 1], [0xffu8; 1])?;

        Ok(())
    }
}
