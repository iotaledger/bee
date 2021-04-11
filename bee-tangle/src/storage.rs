// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{metadata::MessageMetadata, unconfirmed_message::UnconfirmedMessage};

use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    prelude::HashedIndex,
    Message, MessageId,
};
use bee_snapshot::storage::StorageBackend as SnapshotStorageBackend;
use bee_storage::{
    access::{Batch, BatchBuilder, Fetch, Insert},
    backend,
};

pub trait StorageBackend:
    backend::StorageBackend
    + BatchBuilder
    + Batch<MessageId, Message>
    + Batch<MessageId, MessageMetadata>
    + Batch<(MessageId, MessageId), ()>
    + Batch<MilestoneIndex, Milestone>
    + Batch<(MilestoneIndex, UnconfirmedMessage), ()>
    + Batch<(HashedIndex, MessageId), ()>
    + Insert<MessageId, Message>
    + Insert<MessageId, MessageMetadata>
    + Insert<(MessageId, MessageId), ()>
    + Insert<MilestoneIndex, Milestone>
    + Fetch<MessageId, Message>
    + Fetch<MessageId, MessageMetadata>
    + Fetch<MessageId, Vec<MessageId>>
    + Fetch<MilestoneIndex, Milestone>
    + Fetch<MilestoneIndex, Vec<UnconfirmedMessage>>
    + SnapshotStorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + BatchBuilder
        + Batch<MessageId, Message>
        + Batch<MessageId, MessageMetadata>
        + Batch<(MessageId, MessageId), ()>
        + Batch<MilestoneIndex, Milestone>
        + Batch<(MilestoneIndex, UnconfirmedMessage), ()>
        + Batch<(HashedIndex, MessageId), ()>
        + Insert<MessageId, Message>
        + Insert<MessageId, MessageMetadata>
        + Insert<(MessageId, MessageId), ()>
        + Insert<MilestoneIndex, Milestone>
        + Fetch<MessageId, Message>
        + Fetch<MessageId, MessageMetadata>
        + Fetch<MessageId, Vec<MessageId>>
        + Fetch<MilestoneIndex, Milestone>
        + Fetch<MilestoneIndex, Vec<UnconfirmedMessage>>
        + SnapshotStorageBackend
{
}
