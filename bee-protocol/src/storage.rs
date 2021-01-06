// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{tangle::MessageMetadata, Milestone, MilestoneIndex};

use bee_message::{payload::indexation::HashedIndex, Message, MessageId};
use bee_storage::{
    access::{Fetch, Insert},
    storage,
};

pub trait Backend: storage::Backend
    + Insert<MessageId, Message>
    + Insert<MessageId, MessageMetadata>
    //+ Insert<MessageId, Vec<MessageId>>
    + Insert<(MessageId, MessageId), ()>
    + Insert<MilestoneIndex, Milestone>
    + Insert<(HashedIndex, MessageId), ()>
    + Fetch<MessageId, Message>
    + Fetch<MessageId, MessageMetadata>
    + Fetch<MessageId, Vec<MessageId>>
    + Fetch<MilestoneIndex, Milestone> {}

impl<T> Backend for T where
    T: storage::Backend
        + Insert<MessageId, Message>
        + Insert<MessageId, MessageMetadata>
        + Insert<(MessageId, MessageId), ()>
        + Insert<MilestoneIndex, Milestone>
        + Insert<(HashedIndex, MessageId), ()>
        + Fetch<MessageId, Message>
        + Fetch<MessageId, MessageMetadata>
        + Fetch<MessageId, Vec<MessageId>>
        + Fetch<MessageId, Vec<MessageId>>
        + Fetch<MilestoneIndex, Milestone>
{
}
