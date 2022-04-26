// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    milestone::MilestoneIndex,
    payload::milestone::{MilestoneId, MilestonePayload},
    Message, MessageId,
};
use bee_storage::{
    access::{Fetch, Insert},
    backend,
};

use crate::{metadata::MessageMetadata, solid_entry_point::SolidEntryPoint};

/// A blanket-implemented helper trait for the storage layer.
pub trait StorageBackend:
    backend::StorageBackend
    + Insert<MessageId, Message>
    + Insert<MessageId, MessageMetadata>
    + Insert<(MessageId, MessageId), ()>
    + Insert<MilestoneIndex, MilestoneId>
    + Insert<MilestoneId, MilestonePayload>
    + Insert<SolidEntryPoint, MilestoneIndex>
    + Fetch<MessageId, Message>
    + Fetch<MessageId, MessageMetadata>
    + Fetch<MessageId, Vec<MessageId>>
    + Fetch<MilestoneIndex, MilestoneId>
    + Fetch<MilestoneId, MilestonePayload>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Insert<MessageId, Message>
        + Insert<MessageId, MessageMetadata>
        + Insert<(MessageId, MessageId), ()>
        + Insert<MilestoneIndex, MilestoneId>
        + Insert<MilestoneId, MilestonePayload>
        + Insert<SolidEntryPoint, MilestoneIndex>
        + Fetch<MessageId, Message>
        + Fetch<MessageId, MessageMetadata>
        + Fetch<MessageId, Vec<MessageId>>
        + Fetch<MilestoneIndex, MilestoneId>
        + Fetch<MilestoneId, MilestonePayload>
{
}
