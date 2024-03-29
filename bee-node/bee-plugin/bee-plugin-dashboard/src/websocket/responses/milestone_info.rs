// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_tangle::event::LatestMilestoneChanged;
use serde::Serialize;

use crate::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct MilestoneInfoResponse {
    #[serde(rename = "id")] // not consistent with with milestone.rs
    block_id: String,
}

pub(crate) fn forward(latest_milestone: LatestMilestoneChanged) -> WsEvent {
    WsEvent::new(
        WsTopic::MilestoneInfo,
        WsEventInner::MilestoneInfo(MilestoneInfoResponse {
            block_id: latest_milestone.milestone.block_id().to_string(),
        }),
    )
}
