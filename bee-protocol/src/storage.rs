// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::storage::StorageBackend as LedgerStorageBackend;
use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    payload::indexation::HashedIndex,
    solid_entry_point::SolidEntryPoint,
    Message, MessageId,
};
use bee_snapshot::info::SnapshotInfo;
use bee_storage::{
    access::{AsStream, Batch, Fetch, Insert},
    backend,
};
use bee_tangle::{metadata::MessageMetadata, unconfirmed_message::UnconfirmedMessage};

pub trait StorageBackend:
    backend::StorageBackend
    + Batch<(MilestoneIndex, UnconfirmedMessage), ()>
    + Insert<MessageId, Message>
    + Insert<MessageId, MessageMetadata>
    + Insert<(MessageId, MessageId), ()>
    + Insert<MilestoneIndex, Milestone>
    + Insert<(HashedIndex, MessageId), ()>
    + Insert<(MilestoneIndex, UnconfirmedMessage), ()>
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
        + Batch<(MilestoneIndex, UnconfirmedMessage), ()>
        + Insert<MessageId, Message>
        + Insert<MessageId, MessageMetadata>
        + Insert<(MessageId, MessageId), ()>
        + Insert<MilestoneIndex, Milestone>
        + Insert<(HashedIndex, MessageId), ()>
        + Insert<(MilestoneIndex, UnconfirmedMessage), ()>
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
