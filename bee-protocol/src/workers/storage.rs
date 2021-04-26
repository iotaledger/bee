// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::{consensus::storage::StorageBackend as LedgerStorageBackend, snapshot::info::SnapshotInfo};
use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    payload::indexation::PaddedIndex,
    Message, MessageId,
};
use bee_storage::{
    access::{AsStream, Batch, Fetch, Insert},
    backend,
};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

pub trait StorageBackend:
    backend::StorageBackend
    + Batch<(MilestoneIndex, UnreferencedMessage), ()>
    + Insert<MessageId, Message>
    + Insert<MessageId, MessageMetadata>
    + Insert<(MessageId, MessageId), ()>
    + Insert<MilestoneIndex, Milestone>
    + Insert<(PaddedIndex, MessageId), ()>
    + Insert<(MilestoneIndex, UnreferencedMessage), ()>
    + Fetch<MessageId, Message>
    + Fetch<MessageId, MessageMetadata>
    + Fetch<MessageId, Vec<MessageId>>
    + Fetch<MilestoneIndex, Milestone>
    + Fetch<(), SnapshotInfo>
    + for<'a> AsStream<'a, SolidEntryPoint, MilestoneIndex>
    + LedgerStorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Batch<(MilestoneIndex, UnreferencedMessage), ()>
        + Insert<MessageId, Message>
        + Insert<MessageId, MessageMetadata>
        + Insert<(MessageId, MessageId), ()>
        + Insert<MilestoneIndex, Milestone>
        + Insert<(PaddedIndex, MessageId), ()>
        + Insert<(MilestoneIndex, UnreferencedMessage), ()>
        + Fetch<MessageId, Message>
        + Fetch<MessageId, MessageMetadata>
        + Fetch<MessageId, Vec<MessageId>>
        + Fetch<MessageId, Vec<MessageId>>
        + Fetch<MilestoneIndex, Milestone>
        + Fetch<(), SnapshotInfo>
        + for<'a> AsStream<'a, SolidEntryPoint, MilestoneIndex>
        + LedgerStorageBackend
{
}
