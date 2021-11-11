// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

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
use bee_storage::access::Exist;
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

impl Exist<MessageId, Message> for Storage {
    fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?, message_id)?
            .is_some())
    }
}

impl Exist<MessageId, MessageMetadata> for Storage {
    fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MESSAGE_ID_TO_METADATA)?, message_id)?
            .is_some())
    }
}

impl Exist<(MessageId, MessageId), ()> for Storage {
    fn exist(&self, (parent, child): &(MessageId, MessageId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)?, key)?
            .is_some())
    }
}

impl Exist<(PaddedIndex, MessageId), ()> for Storage {
    fn exist(&self, (index, message_id): &(PaddedIndex, MessageId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_INDEX_TO_MESSAGE_ID)?, key)?
            .is_some())
    }
}

impl Exist<OutputId, CreatedOutput> for Storage {
    fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)?, output_id.pack_new())?
            .is_some())
    }
}

impl Exist<OutputId, ConsumedOutput> for Storage {
    fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)?, output_id.pack_new())?
            .is_some())
    }
}

impl Exist<Unspent, ()> for Storage {
    fn exist(&self, unspent: &Unspent) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_OUTPUT_ID_UNSPENT)?, unspent.pack_new())?
            .is_some())
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
            .get_cf(self.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)?, key)?
            .is_some())
    }
}

impl Exist<(AliasAddress, OutputId), ()> for Storage {
    fn exist(&self, (address, output_id): &(AliasAddress, OutputId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_ALIAS_ADDRESS_TO_OUTPUT_ID)?, key)?
            .is_some())
    }
}

impl Exist<(NftAddress, OutputId), ()> for Storage {
    fn exist(&self, (address, output_id): &(NftAddress, OutputId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_NFT_ADDRESS_TO_OUTPUT_ID)?, key)?
            .is_some())
    }
}

impl Exist<(), LedgerIndex> for Storage {
    fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self.inner.get_cf(self.cf_handle(CF_LEDGER_INDEX)?, [0x00u8])?.is_some())
    }
}

impl Exist<MilestoneIndex, Milestone> for Storage {
    fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE)?, index.pack_new())?
            .is_some())
    }
}

impl Exist<(), SnapshotInfo> for Storage {
    fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_SNAPSHOT_INFO)?, [0x00u8])?
            .is_some())
    }
}

impl Exist<SolidEntryPoint, MilestoneIndex> for Storage {
    fn exist(&self, sep: &SolidEntryPoint) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?, sep.pack_new())?
            .is_some())
    }
}

impl Exist<MilestoneIndex, OutputDiff> for Storage {
    fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)?, index.pack_new())?
            .is_some())
    }
}

impl Exist<Address, Balance> for Storage {
    fn exist(&self, address: &Address) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_ADDRESS_TO_BALANCE)?, address.pack_new())?
            .is_some())
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
            .get_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?, key)?
            .is_some())
    }
}

impl Exist<(MilestoneIndex, Receipt), ()> for Storage {
    fn exist(&self, (index, receipt): &(MilestoneIndex, Receipt)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(&receipt.pack_new());

        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)?, key)?
            .is_some())
    }
}

impl Exist<(bool, TreasuryOutput), ()> for Storage {
    fn exist(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = spent.pack_new();
        key.extend_from_slice(&output.pack_new());

        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)?, key)?
            .is_some())
    }
}
