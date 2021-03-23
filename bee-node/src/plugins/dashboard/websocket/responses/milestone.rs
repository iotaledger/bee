// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

use bee_protocol::workers::event::LatestMilestoneChanged;

use serde::Serialize;

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
