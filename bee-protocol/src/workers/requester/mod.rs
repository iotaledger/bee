// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod message;
mod milestone;

pub use milestone::RequestedMilestones;

pub use self::message::{request_message, MessageRequesterWorker, MessageRequesterWorkerEvent, RequestedMessages};
pub(crate) use self::milestone::{
    request_latest_milestone, request_milestone, MilestoneRequesterWorker, MilestoneRequesterWorkerEvent,
};
