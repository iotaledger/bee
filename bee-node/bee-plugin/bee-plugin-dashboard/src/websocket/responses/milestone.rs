// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_tangle::event::LatestMilestoneChanged;
use serde::Serialize;

use crate::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct MilestoneResponse {
    #[serde(rename = "messageID")] // shouldn't it be messageId to be consistent with the REST API?
    message_id: String,
    index: u32,
}

pub(crate) fn forward(latest_milestone: LatestMilestoneChanged) -> WsEvent {
    WsEvent::new(
        WsTopic::Milestone,
        WsEventInner::Milestone(MilestoneResponse {
            message_id: latest_milestone.milestone.message_id().to_string(),
            index: *latest_milestone.index,
        }),
    )
}
