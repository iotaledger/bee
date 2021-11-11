// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Insert access operations.

use crate::storage::Storage;

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
use bee_storage::{access::Insert, backend::StorageBackend, system::System};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

macro_rules! impl_insert {
    ($key:ty, $value:ty, $field:ident) => {
        impl Insert<$key, $value> for Storage {
            fn insert(&self, k: &$key, v: &$value) -> Result<(), <Self as StorageBackend>::Error> {
                self.inner.write()?.$field.insert(k, v);

                Ok(())
            }
        }
    };
}

impl_insert!(u8, System, system);
impl_insert!(MessageId, Message, message_id_to_message);
impl_insert!(MessageId, MessageMetadata, message_id_to_metadata);
impl_insert!((MessageId, MessageId), (), message_id_to_message_id);
impl_insert!((PaddedIndex, MessageId), (), index_to_message_id);
impl_insert!(OutputId, CreatedOutput, output_id_to_created_output);
impl_insert!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_insert!(Unspent, (), output_id_unspent);
impl_insert!((Ed25519Address, OutputId), (), ed25519_address_to_output_id);
impl_insert!((AliasAddress, OutputId), (), alias_address_to_output_id);
impl_insert!((NftAddress, OutputId), (), nft_address_to_output_id);
impl_insert!((), LedgerIndex, ledger_index);
impl_insert!(MilestoneIndex, Milestone, milestone_index_to_milestone);
impl_insert!((), SnapshotInfo, snapshot_info);
impl_insert!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_insert!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_insert!(Address, Balance, address_to_balance);
impl_insert!(
    (MilestoneIndex, UnreferencedMessage),
    (),
    milestone_index_to_unreferenced_message
);
impl_insert!((MilestoneIndex, Receipt), (), milestone_index_to_receipt);
impl_insert!((bool, TreasuryOutput), (), spent_to_treasury_output);
