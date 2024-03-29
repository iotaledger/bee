// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    address::Ed25519Address,
    output::OutputId,
    payload::milestone::{MilestoneId, MilestoneIndex, MilestonePayload},
    Block, BlockId,
};
use bee_ledger::types::{
    snapshot::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent,
};
use bee_storage::access::Truncate;
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

macro_rules! impl_truncate {
    ($key:ty, $value:ty, $cf:expr) => {
        impl Truncate<$key, $value> for Storage {
            fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
                let cf_handle = self.cf_handle($cf)?;

                let mut iter = self.inner.raw_iterator_cf(cf_handle);

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
                    // `delete_range_cf` excludes the last key in the range so a byte is added to be sure the last key
                    // is included.
                    last.push(u8::MAX);
                    last
                } else {
                    // There are no keys to remove.
                    return Ok(());
                };

                self.inner.delete_range_cf(cf_handle, first, last)?;

                Ok(())
            }
        }
    };
}

impl_truncate!(BlockId, Block, CF_BLOCK_ID_TO_BLOCK);
impl_truncate!((BlockId, BlockId), (), CF_BLOCK_ID_TO_BLOCK_ID);
impl_truncate!(OutputId, CreatedOutput, CF_OUTPUT_ID_TO_CREATED_OUTPUT);
impl_truncate!(OutputId, ConsumedOutput, CF_OUTPUT_ID_TO_CONSUMED_OUTPUT);
impl_truncate!(Unspent, (), CF_OUTPUT_ID_UNSPENT);
impl_truncate!((Ed25519Address, OutputId), (), CF_ED25519_ADDRESS_TO_OUTPUT_ID);
impl_truncate!((), LedgerIndex, CF_LEDGER_INDEX);
impl_truncate!(
    MilestoneIndex,
    MilestoneMetadata,
    CF_MILESTONE_INDEX_TO_MILESTONE_METADATA
);
impl_truncate!(MilestoneId, MilestonePayload, CF_MILESTONE_ID_TO_MILESTONE_PAYLOAD);
impl_truncate!((), SnapshotInfo, CF_SNAPSHOT_INFO);
impl_truncate!(SolidEntryPoint, MilestoneIndex, CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX);
impl_truncate!(MilestoneIndex, OutputDiff, CF_MILESTONE_INDEX_TO_OUTPUT_DIFF);
impl_truncate!(
    (MilestoneIndex, UnreferencedBlock),
    (),
    CF_MILESTONE_INDEX_TO_UNREFERENCED_BLOCK
);
impl_truncate!((MilestoneIndex, Receipt), (), CF_MILESTONE_INDEX_TO_RECEIPT);
impl_truncate!((bool, TreasuryOutput), (), CF_SPENT_TO_TREASURY_OUTPUT);

impl Truncate<BlockId, BlockMetadata> for Storage {
    fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
        let guard = self.locks.block_id_to_metadata.read();

        let cf_handle = self.cf_handle(CF_BLOCK_ID_TO_METADATA)?;

        let mut iter = self.inner.raw_iterator_cf(cf_handle);

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
            // `delete_range_cf` excludes the last key in the range so a byte is added to be sure the last key is
            // included.
            last.push(u8::MAX);
            last
        } else {
            // There are no keys to remove.
            return Ok(());
        };

        self.inner.delete_range_cf(cf_handle, first, last)?;

        drop(guard);

        Ok(())
    }
}
