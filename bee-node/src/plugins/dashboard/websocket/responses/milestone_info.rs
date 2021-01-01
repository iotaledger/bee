// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

use bee_protocol::event::LatestMilestoneChanged;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct MilestoneInfoResponse {
    #[serde(rename = "id")] // not consistent with with milestone.rs
    message_id: String,
}

pub(crate) fn forward(latest_milestone: LatestMilestoneChanged) -> WsEvent {
    WsEvent::new(
        WsTopic::MilestoneInfo,
        WsEventInner::MilestoneInfo(MilestoneInfoResponse {
            message_id: latest_milestone.milestone.message_id().to_string(),
        }),
    )
}
