// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Exist access operations.

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
use bee_storage::{access::Exist, backend::StorageBackend};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

macro_rules! impl_exist {
    ($key:ty, $value:ty, $field:ident) => {
        impl Exist<$key, $value> for Storage {
            fn exist(&self, k: &$key) -> Result<bool, <Self as StorageBackend>::Error> {
                Ok(self.inner.read()?.$field.exist(k))
            }
        }
    };
}

impl_exist!(MessageId, Message, message_id_to_message);
impl_exist!(MessageId, MessageMetadata, message_id_to_metadata);
impl_exist!((MessageId, MessageId), (), message_id_to_message_id);
impl_exist!((PaddedIndex, MessageId), (), index_to_message_id);
impl_exist!(OutputId, CreatedOutput, output_id_to_created_output);
impl_exist!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_exist!(Unspent, (), output_id_unspent);
impl_exist!((Ed25519Address, OutputId), (), ed25519_address_to_output_id);
impl_exist!((AliasAddress, OutputId), (), alias_address_to_output_id);
impl_exist!((NftAddress, OutputId), (), nft_address_to_output_id);
impl_exist!((), LedgerIndex, ledger_index);
impl_exist!(MilestoneIndex, Milestone, milestone_index_to_milestone);
impl_exist!((), SnapshotInfo, snapshot_info);
impl_exist!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_exist!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_exist!(Address, Balance, address_to_balance);
impl_exist!(
    (MilestoneIndex, UnreferencedMessage),
    (),
    milestone_index_to_unreferenced_message
);
impl_exist!((MilestoneIndex, Receipt), (), milestone_index_to_receipt);
impl_exist!((bool, TreasuryOutput), (), spent_to_treasury_output);
