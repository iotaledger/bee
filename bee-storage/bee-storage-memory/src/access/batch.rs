// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Batch access operations.

use crate::{storage::Storage, table::TableBatch};

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
use bee_storage::{
    access::{Batch, BatchBuilder},
    backend::StorageBackend,
};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

/// A writing batch that can be applied atomically.
#[derive(Default)]
pub struct StorageBatch {
    message_id_to_message: TableBatch<MessageId, Message>,
    message_id_to_metadata: TableBatch<MessageId, MessageMetadata>,
    message_id_to_message_id: TableBatch<(MessageId, MessageId), ()>,
    index_to_message_id: TableBatch<(PaddedIndex, MessageId), ()>,
    output_id_to_created_output: TableBatch<OutputId, CreatedOutput>,
    output_id_to_consumed_output: TableBatch<OutputId, ConsumedOutput>,
    output_id_unspent: TableBatch<Unspent, ()>,
    ed25519_address_to_output_id: TableBatch<(Ed25519Address, OutputId), ()>,
    ledger_index: TableBatch<(), LedgerIndex>,
    milestone_index_to_milestone: TableBatch<MilestoneIndex, Milestone>,
    snapshot_info: TableBatch<(), SnapshotInfo>,
    solid_entry_point_to_milestone_index: TableBatch<SolidEntryPoint, MilestoneIndex>,
    milestone_index_to_output_diff: TableBatch<MilestoneIndex, OutputDiff>,
    address_to_balance: TableBatch<Address, Balance>,
    milestone_index_to_unreferenced_message: TableBatch<(MilestoneIndex, UnreferencedMessage), ()>,
    milestone_index_to_receipt: TableBatch<(MilestoneIndex, Receipt), ()>,
    spent_to_treasury_output: TableBatch<(bool, TreasuryOutput), ()>,
}

impl BatchBuilder for Storage {
    type Batch = StorageBatch;

    fn batch_begin() -> Self::Batch {
        Self::Batch::default()
    }

    fn batch_commit(&self, batch: Self::Batch, _durability: bool) -> Result<(), <Self as StorageBackend>::Error> {
        macro_rules! apply_batch {
            ($field:ident) => {
                self.$field.batch_commit(batch.$field)?;
            };
        }

        apply_batch!(message_id_to_message);
        apply_batch!(message_id_to_metadata);
        apply_batch!(message_id_to_message_id);
        apply_batch!(index_to_message_id);
        apply_batch!(output_id_to_created_output);
        apply_batch!(output_id_to_consumed_output);
        apply_batch!(output_id_unspent);
        apply_batch!(ed25519_address_to_output_id);
        apply_batch!(ledger_index);
        apply_batch!(milestone_index_to_milestone);
        apply_batch!(snapshot_info);
        apply_batch!(solid_entry_point_to_milestone_index);
        apply_batch!(milestone_index_to_output_diff);
        apply_batch!(address_to_balance);
        apply_batch!(milestone_index_to_unreferenced_message);
        apply_batch!(milestone_index_to_receipt);
        apply_batch!(spent_to_treasury_output);

        Ok(())
    }
}

macro_rules! impl_batch {
    ($key:ty, $value:ty, $field:ident) => {
        impl Batch<$key, $value> for Storage {
            fn batch_insert(
                &self,
                batch: &mut Self::Batch,
                key: &$key,
                value: &$value,
            ) -> Result<(), <Self as StorageBackend>::Error> {
                batch.$field.insert(key, value);

                Ok(())
            }

            fn batch_delete(&self, batch: &mut Self::Batch, key: &$key) -> Result<(), <Self as StorageBackend>::Error> {
                batch.$field.delete(key);

                Ok(())
            }
        }
    };
}

impl_batch!(MessageId, Message, message_id_to_message);
impl_batch!(MessageId, MessageMetadata, message_id_to_metadata);
impl_batch!((MessageId, MessageId), (), message_id_to_message_id);
impl_batch!((PaddedIndex, MessageId), (), index_to_message_id);
impl_batch!(OutputId, CreatedOutput, output_id_to_created_output);
impl_batch!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_batch!(Unspent, (), output_id_unspent);
impl_batch!((Ed25519Address, OutputId), (), ed25519_address_to_output_id);
impl_batch!((), LedgerIndex, ledger_index);
impl_batch!(MilestoneIndex, Milestone, milestone_index_to_milestone);
impl_batch!((), SnapshotInfo, snapshot_info);
impl_batch!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_batch!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_batch!(Address, Balance, address_to_balance);
impl_batch!(
    (MilestoneIndex, UnreferencedMessage),
    (),
    milestone_index_to_unreferenced_message
);
impl_batch!((MilestoneIndex, Receipt), (), milestone_index_to_receipt);
impl_batch!((bool, TreasuryOutput), (), spent_to_treasury_output);
