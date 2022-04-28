// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Truncate access operations.

use bee_ledger::types::{
    snapshot::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent,
};
use bee_message::{
    address::Ed25519Address,
    output::OutputId,
    payload::milestone::{MilestoneId, MilestoneIndex, MilestonePayload},
    Message, MessageId,
};
use bee_storage::{access::Truncate, backend::StorageBackend};
use bee_tangle::{
    message_metadata::MessageMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_message::UnreferencedMessage,
};

use crate::storage::Storage;

macro_rules! impl_truncate {
    ($key:ty, $value:ty, $field:ident) => {
        impl Truncate<$key, $value> for Storage {
            fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
                self.inner.write()?.$field.truncate();

                Ok(())
            }
        }
    };
}

impl_truncate!(MessageId, Message, message_id_to_message);
impl_truncate!(MessageId, MessageMetadata, message_id_to_metadata);
impl_truncate!((MessageId, MessageId), (), message_id_to_message_id);
impl_truncate!(OutputId, CreatedOutput, output_id_to_created_output);
impl_truncate!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_truncate!(Unspent, (), output_id_unspent);
impl_truncate!((Ed25519Address, OutputId), (), ed25519_address_to_output_id);
impl_truncate!((), LedgerIndex, ledger_index);
impl_truncate!(MilestoneIndex, MilestoneMetadata, milestone_index_to_milestone_metadata);
impl_truncate!(MilestoneId, MilestonePayload, milestone_id_to_milestone_payload);
impl_truncate!((), SnapshotInfo, snapshot_info);
impl_truncate!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_truncate!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_truncate!(
    (MilestoneIndex, UnreferencedMessage),
    (),
    milestone_index_to_unreferenced_message
);
impl_truncate!((MilestoneIndex, Receipt), (), milestone_index_to_receipt);
impl_truncate!((bool, TreasuryOutput), (), spent_to_treasury_output);
