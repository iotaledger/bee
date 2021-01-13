// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage::*};

use bee_common::packable::Packable;
use bee_ledger::model::{Diff, Output, Spent};
use bee_message::{
    ledger_index::LedgerIndex,
    milestone::{Milestone, MilestoneIndex},
    payload::{
        indexation::{HashedIndex, HASHED_INDEX_LENGTH},
        transaction::{Ed25519Address, OutputId, ED25519_ADDRESS_LENGTH, OUTPUT_ID_LENGTH},
    },
    solid_entry_point::SolidEntryPoint,
    Message, MessageId, MESSAGE_ID_LENGTH,
};
use bee_snapshot::info::SnapshotInfo;
use bee_storage::access::Fetch;
use bee_tangle::metadata::MessageMetadata;

use std::convert::TryInto;

#[async_trait::async_trait]
impl Fetch<MessageId, Message> for Storage {
    async fn fetch(&self, message_id: &MessageId) -> Result<Option<Message>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE))?;

        if let Some(res) = self.inner.get_cf(&cf, message_id)? {
            // Unpacking from storage is fine.
            Ok(Some(Message::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<MessageId, MessageMetadata> for Storage {
    async fn fetch(&self, message_id: &MessageId) -> Result<Option<MessageMetadata>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_METADATA)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_METADATA))?;

        if let Some(res) = self.inner.get_cf(&cf, message_id)? {
            // Unpacking from storage is fine.
            Ok(Some(MessageMetadata::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<MessageId, Vec<MessageId>> for Storage {
    async fn fetch(&self, parent: &MessageId) -> Result<Option<Vec<MessageId>>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE_ID))?;

        Ok(Some(
            self.inner
                .prefix_iterator_cf(&cf, parent)
                .map(|(key, _)| {
                    let (_, child) = key.split_at(MESSAGE_ID_LENGTH);
                    // Unpacking from storage is fine.
                    let child: [u8; MESSAGE_ID_LENGTH] = child.try_into().unwrap();
                    MessageId::from(child)
                })
                .take(self.config.fetch_edge_limit)
                .collect(),
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<HashedIndex, Vec<MessageId>> for Storage {
    async fn fetch(&self, index: &HashedIndex) -> Result<Option<Vec<MessageId>>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_INDEX_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_INDEX_TO_MESSAGE_ID))?;

        Ok(Some(
            self.inner
                .prefix_iterator_cf(&cf, index)
                .map(|(key, _)| {
                    let (_, message_id) = key.split_at(HASHED_INDEX_LENGTH);
                    // Unpacking from storage is fine.
                    let message_id: [u8; MESSAGE_ID_LENGTH] = message_id.try_into().unwrap();
                    MessageId::from(message_id)
                })
                .take(self.config.fetch_index_limit)
                .collect(),
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<OutputId, Output> for Storage {
    async fn fetch(&self, output_id: &OutputId) -> Result<Option<Output>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_OUTPUT))?;

        if let Some(res) = self.inner.get_cf(&cf, output_id.pack_new())? {
            // Unpacking from storage is fine.
            Ok(Some(Output::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<OutputId, Spent> for Storage {
    async fn fetch(&self, output_id: &OutputId) -> Result<Option<Spent>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_SPENT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_SPENT))?;

        if let Some(res) = self.inner.get_cf(&cf, output_id.pack_new())? {
            // Unpacking from storage is fine.
            Ok(Some(Spent::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<Ed25519Address, Vec<OutputId>> for Storage {
    async fn fetch(&self, address: &Ed25519Address) -> Result<Option<Vec<OutputId>>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)
            .ok_or(Error::UnknownCf(CF_ED25519_ADDRESS_TO_OUTPUT_ID))?;

        Ok(Some(
            self.inner
                .prefix_iterator_cf(&cf, address)
                .map(|(key, _)| {
                    let (_, output_id) = key.split_at(ED25519_ADDRESS_LENGTH);
                    // Unpacking from storage is fine.
                    From::<[u8; OUTPUT_ID_LENGTH]>::from(output_id.try_into().unwrap())
                })
                .take(self.config.fetch_output_id_limit)
                .collect(),
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<(), LedgerIndex> for Storage {
    async fn fetch(&self, (): &()) -> Result<Option<LedgerIndex>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_LEDGER_INDEX)
            .ok_or(Error::UnknownCf(CF_LEDGER_INDEX))?;

        if let Some(res) = self.inner.get_cf(&cf, [0x00u8])? {
            // Unpacking from storage is fine.
            Ok(Some(LedgerIndex::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<MilestoneIndex, Milestone> for Storage {
    async fn fetch(&self, index: &MilestoneIndex) -> Result<Option<Milestone>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_MILESTONE))?;

        if let Some(res) = self.inner.get_cf(&cf, index.pack_new())? {
            // Unpacking from storage is fine.
            Ok(Some(Milestone::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<(), SnapshotInfo> for Storage {
    async fn fetch(&self, (): &()) -> Result<Option<SnapshotInfo>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SNAPSHOT_INFO)
            .ok_or(Error::UnknownCf(CF_SNAPSHOT_INFO))?;

        if let Some(res) = self.inner.get_cf(&cf, [0x00u8])? {
            // Unpacking from storage is fine.
            Ok(Some(SnapshotInfo::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn fetch(&self, sep: &SolidEntryPoint) -> Result<Option<MilestoneIndex>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)
            .ok_or(Error::UnknownCf(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX))?;

        if let Some(res) = self.inner.get_cf(&cf, sep.pack_new())? {
            // Unpacking from storage is fine.
            Ok(Some(MilestoneIndex::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<MilestoneIndex, Diff> for Storage {
    async fn fetch(&self, index: &MilestoneIndex) -> Result<Option<Diff>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_DIFF)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_DIFF))?;

        if let Some(res) = self.inner.get_cf(&cf, index.pack_new())? {
            // Unpacking from storage is fine.
            Ok(Some(Diff::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}
