// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage::*, system::System};

use bee_common::packable::Packable;
use bee_ledger::{
    snapshot::info::SnapshotInfo,
    types::{Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput},
};
use bee_message::{
    address::{Address, Ed25519Address, ED25519_ADDRESS_LENGTH},
    milestone::{Milestone, MilestoneIndex},
    output::{OutputId, OUTPUT_ID_LENGTH},
    payload::indexation::{HashedIndex, HASHED_INDEX_LENGTH},
    Message, MessageId, MESSAGE_ID_LENGTH,
};
use bee_storage::access::Fetch;
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unconfirmed_message::UnconfirmedMessage,
};

use std::convert::{TryFrom, TryInto};

#[async_trait::async_trait]
impl Fetch<u8, System> for Storage {
    async fn fetch(&self, key: &u8) -> Result<Option<System>, <Self as StorageBackend>::Error> {
        let cf = self.inner.cf_handle(CF_SYSTEM).ok_or(Error::UnknownCf(CF_SYSTEM))?;

        if let Some(res) = self.inner.get_cf(cf, [*key])? {
            // Unpacking from storage is fine.
            Ok(Some(System::unpack_unchecked(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<MessageId, Message> for Storage {
    async fn fetch(&self, message_id: &MessageId) -> Result<Option<Message>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE))?;

        if let Some(res) = self.inner.get_cf(cf, message_id)? {
            // Unpacking from storage is fine.
            Ok(Some(Message::unpack_unchecked(&mut res.as_slice()).unwrap()))
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

        if let Some(res) = self.inner.get_cf(cf, message_id)? {
            // Unpacking from storage is fine.
            Ok(Some(MessageMetadata::unpack_unchecked(&mut res.as_slice()).unwrap()))
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
                .prefix_iterator_cf(cf, parent)
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
                .prefix_iterator_cf(cf, index)
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
impl Fetch<OutputId, CreatedOutput> for Storage {
    async fn fetch(&self, output_id: &OutputId) -> Result<Option<CreatedOutput>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CREATED_OUTPUT))?;

        if let Some(res) = self.inner.get_cf(cf, output_id.pack_new())? {
            // Unpacking from storage is fine.
            Ok(Some(CreatedOutput::unpack_unchecked(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<OutputId, ConsumedOutput> for Storage {
    async fn fetch(&self, output_id: &OutputId) -> Result<Option<ConsumedOutput>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT))?;

        if let Some(res) = self.inner.get_cf(cf, output_id.pack_new())? {
            // Unpacking from storage is fine.
            Ok(Some(ConsumedOutput::unpack_unchecked(&mut res.as_slice()).unwrap()))
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
                .prefix_iterator_cf(cf, address)
                .map(|(key, _)| {
                    let (_, output_id) = key.split_at(ED25519_ADDRESS_LENGTH);
                    // Unpacking from storage is fine.
                    TryFrom::<[u8; OUTPUT_ID_LENGTH]>::try_from(output_id.try_into().unwrap()).unwrap()
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

        if let Some(res) = self.inner.get_cf(cf, [0x00u8])? {
            // Unpacking from storage is fine.
            Ok(Some(LedgerIndex::unpack_unchecked(&mut res.as_slice()).unwrap()))
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

        if let Some(res) = self.inner.get_cf(cf, index.pack_new())? {
            // Unpacking from storage is fine.
            Ok(Some(Milestone::unpack_unchecked(&mut res.as_slice()).unwrap()))
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

        if let Some(res) = self.inner.get_cf(cf, [0x00u8])? {
            // Unpacking from storage is fine.
            Ok(Some(SnapshotInfo::unpack_unchecked(&mut res.as_slice()).unwrap()))
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

        if let Some(res) = self.inner.get_cf(cf, sep.pack_new())? {
            // Unpacking from storage is fine.
            Ok(Some(MilestoneIndex::unpack_unchecked(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<MilestoneIndex, OutputDiff> for Storage {
    async fn fetch(&self, index: &MilestoneIndex) -> Result<Option<OutputDiff>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF))?;

        if let Some(res) = self.inner.get_cf(cf, index.pack_new())? {
            // Unpacking from storage is fine.
            Ok(Some(OutputDiff::unpack_unchecked(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<Address, Balance> for Storage {
    async fn fetch(&self, address: &Address) -> Result<Option<Balance>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ADDRESS_TO_BALANCE)
            .ok_or(Error::UnknownCf(CF_ADDRESS_TO_BALANCE))?;

        if let Some(res) = self.inner.get_cf(cf, address.pack_new())? {
            // Unpacking from storage is fine.
            Ok(Some(Balance::unpack_unchecked(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<MilestoneIndex, Vec<UnconfirmedMessage>> for Storage {
    async fn fetch(
        &self,
        index: &MilestoneIndex,
    ) -> Result<Option<Vec<UnconfirmedMessage>>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE))?;

        Ok(Some(
            self.inner
                .prefix_iterator_cf(cf, index.pack_new())
                .map(|(key, _)| {
                    let (_, unconfirmed_message) = key.split_at(std::mem::size_of::<MilestoneIndex>());
                    // Unpacking from storage is fine.
                    let unconfirmed_message: [u8; MESSAGE_ID_LENGTH] = unconfirmed_message.try_into().unwrap();
                    UnconfirmedMessage::from(MessageId::from(unconfirmed_message))
                })
                .collect(),
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<MilestoneIndex, Vec<Receipt>> for Storage {
    async fn fetch(&self, index: &MilestoneIndex) -> Result<Option<Vec<Receipt>>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_RECEIPT))?;

        Ok(Some(
            self.inner
                .prefix_iterator_cf(cf, index.pack_new())
                .map(|(mut key, _)| {
                    let (_, receipt) = key.split_at_mut(std::mem::size_of::<MilestoneIndex>());
                    // Unpacking from storage is fine.
                    Receipt::unpack_unchecked(&mut receipt.as_ref()).unwrap()
                })
                .collect(),
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<bool, Vec<TreasuryOutput>> for Storage {
    async fn fetch(&self, spent: &bool) -> Result<Option<Vec<TreasuryOutput>>, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)
            .ok_or(Error::UnknownCf(CF_SPENT_TO_TREASURY_OUTPUT))?;

        Ok(Some(
            self.inner
                .prefix_iterator_cf(cf, spent.pack_new())
                .map(|(mut key, _)| {
                    let (_, output) = key.split_at_mut(std::mem::size_of::<bool>());
                    // Unpacking from storage is fine.
                    TreasuryOutput::unpack_unchecked(&mut output.as_ref()).unwrap()
                })
                .collect(),
        ))
    }
}
