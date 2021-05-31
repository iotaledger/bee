// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Truncate access operations.

use crate::{storage::Storage, trees::*};

use bee_ledger::types::{
    snapshot::SnapshotInfo, Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput,
    Unspent,
};
use bee_message::{
    address::{Address, Ed25519Address},
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::indexation::PaddedIndex,
    Message, MessageId,
};
use bee_storage::{access::Truncate, backend::StorageBackend};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

fn truncate(storage: &Storage, tree_str: &'static str) -> Result<(), <Storage as StorageBackend>::Error> {
    storage.inner.drop_tree(tree_str)?;

    Ok(())
}

macro_rules! impl_truncate {
    ($key:ty, $value:ty, $cf:expr) => {
        #[async_trait::async_trait]
        impl Truncate<$key, $value> for Storage {
            async fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
                truncate(self, $cf)
            }
        }
    };
}

impl_truncate!(MessageId, Message, TREE_MESSAGE_ID_TO_MESSAGE);
impl_truncate!(MessageId, MessageMetadata, TREE_MESSAGE_ID_TO_METADATA);
impl_truncate!((MessageId, MessageId), (), TREE_MESSAGE_ID_TO_MESSAGE_ID);
impl_truncate!((PaddedIndex, MessageId), (), TREE_INDEX_TO_MESSAGE_ID);
impl_truncate!(OutputId, CreatedOutput, TREE_OUTPUT_ID_TO_CREATED_OUTPUT);
impl_truncate!(OutputId, ConsumedOutput, TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT);
impl_truncate!(Unspent, (), TREE_OUTPUT_ID_UNSPENT);
impl_truncate!((Ed25519Address, OutputId), (), TREE_ED25519_ADDRESS_TO_OUTPUT_ID);
impl_truncate!((), LedgerIndex, TREE_LEDGER_INDEX);
impl_truncate!(MilestoneIndex, Milestone, TREE_MILESTONE_INDEX_TO_MILESTONE);
impl_truncate!((), SnapshotInfo, TREE_SNAPSHOT_INFO);
impl_truncate!(
    SolidEntryPoint,
    MilestoneIndex,
    TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX
);
impl_truncate!(MilestoneIndex, OutputDiff, TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF);
impl_truncate!(Address, Balance, TREE_ADDRESS_TO_BALANCE);
impl_truncate!(
    (MilestoneIndex, UnreferencedMessage),
    (),
    TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE
);
impl_truncate!((MilestoneIndex, Receipt), (), TREE_MILESTONE_INDEX_TO_RECEIPT);
impl_truncate!((bool, TreasuryOutput), (), TREE_SPENT_TO_TREASURY_OUTPUT);
