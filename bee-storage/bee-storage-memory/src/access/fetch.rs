// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Fetch access operations.

use bee_ledger::types::{
    snapshot::info::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput,
};
use bee_message::{
    address::Ed25519Address,
    milestone::MilestoneIndex,
    output::OutputId,
    payload::milestone::{MilestoneId, MilestonePayload},
    Message, MessageId,
};
use bee_storage::{access::Fetch, backend::StorageBackend, system::System};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

use crate::storage::Storage;

macro_rules! impl_fetch {
    ($key:ty, $value:ty, $field:ident) => {
        impl Fetch<$key, $value> for Storage {
            fn fetch(&self, k: &$key) -> Result<Option<$value>, <Self as StorageBackend>::Error> {
                Ok(self.inner.read()?.$field.fetch(k))
            }
        }
    };
}

impl_fetch!(u8, System, system);
impl_fetch!(MessageId, Message, message_id_to_message);
impl_fetch!(MessageId, MessageMetadata, message_id_to_metadata);
impl_fetch!(MessageId, Vec<MessageId>, message_id_to_message_id);
impl_fetch!(OutputId, CreatedOutput, output_id_to_created_output);
impl_fetch!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_fetch!(Ed25519Address, Vec<OutputId>, ed25519_address_to_output_id);
impl_fetch!((), LedgerIndex, ledger_index);
impl_fetch!(MilestoneIndex, MilestoneId, milestone_index_to_milestone_id);
impl_fetch!(MilestoneId, MilestonePayload, milestone_id_to_milestone_payload);
impl_fetch!((), SnapshotInfo, snapshot_info);
impl_fetch!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_fetch!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_fetch!(
    MilestoneIndex,
    Vec<UnreferencedMessage>,
    milestone_index_to_unreferenced_message
);
impl_fetch!(MilestoneIndex, Vec<Receipt>, milestone_index_to_receipt);
impl_fetch!(bool, Vec<TreasuryOutput>, spent_to_treasury_output);
