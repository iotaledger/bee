// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Delete access operations.

use crate::storage::Storage;

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
use bee_storage::{access::Delete, backend::StorageBackend};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

macro_rules! impl_delete {
    ($key:ty, $value:ty, $field:ident) => {
        impl Delete<$key, $value> for Storage {
            fn delete(&self, k: &$key) -> Result<(), <Self as StorageBackend>::Error> {
                self.inner.write()?.$field.delete(k);

                Ok(())
            }
        }
    };
}

impl_delete!(MessageId, Message, message_id_to_message);
impl_delete!(MessageId, MessageMetadata, message_id_to_metadata);
impl_delete!((MessageId, MessageId), (), message_id_to_message_id);
impl_delete!((PaddedIndex, MessageId), (), index_to_message_id);
impl_delete!(OutputId, CreatedOutput, output_id_to_created_output);
impl_delete!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_delete!(Unspent, (), output_id_unspent);
impl_delete!((Ed25519Address, OutputId), (), ed25519_address_to_output_id);
impl_delete!((), LedgerIndex, ledger_index);
impl_delete!(MilestoneIndex, Milestone, milestone_index_to_milestone);
impl_delete!((), SnapshotInfo, snapshot_info);
impl_delete!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_delete!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_delete!(Address, Balance, address_to_balance);
impl_delete!(
    (MilestoneIndex, UnreferencedMessage),
    (),
    milestone_index_to_unreferenced_message
);
impl_delete!((MilestoneIndex, Receipt), (), milestone_index_to_receipt);
impl_delete!((bool, TreasuryOutput), (), spent_to_treasury_output);
