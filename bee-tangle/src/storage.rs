// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{payload::milestone::MilestoneIndex, Message, MessageId};
use bee_storage::{
    access::{Exist, Fetch, Insert, InsertStrict, Update},
    backend,
};

use crate::{
    message_metadata::MessageMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
};

/// A blanket-implemented helper trait for the storage layer.
pub trait StorageBackend:
    backend::StorageBackend
    + Insert<MessageId, Message>
    + Insert<(MessageId, MessageId), ()>
    + Insert<MilestoneIndex, MilestoneMetadata>
    + Insert<SolidEntryPoint, MilestoneIndex>
    + InsertStrict<MessageId, MessageMetadata>
    + Exist<MessageId, Message>
    + Exist<MilestoneIndex, MilestoneMetadata>
    + Fetch<MessageId, Message>
    + Fetch<MessageId, MessageMetadata>
    + Fetch<MessageId, Vec<MessageId>>
    + Fetch<MilestoneIndex, MilestoneMetadata>
    + Update<MessageId, MessageMetadata>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Insert<MessageId, Message>
        + Insert<(MessageId, MessageId), ()>
        + Insert<MilestoneIndex, MilestoneMetadata>
        + Insert<SolidEntryPoint, MilestoneIndex>
        + InsertStrict<MessageId, MessageMetadata>
        + Exist<MessageId, Message>
        + Exist<MilestoneIndex, MilestoneMetadata>
        + Fetch<MessageId, Message>
        + Fetch<MessageId, MessageMetadata>
        + Fetch<MessageId, Vec<MessageId>>
        + Fetch<MilestoneIndex, MilestoneMetadata>
        + Update<MessageId, MessageMetadata>
{
}
