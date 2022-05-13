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
    #[serde(rename = "blockID")] // shouldn't it be blockId to be consistent with the REST API?
    block_id: String,
    index: u32,
}

pub(crate) fn forward(latest_milestone: LatestMilestoneChanged) -> WsEvent {
    WsEvent::new(
        WsTopic::Milestone,
        WsEventInner::Milestone(MilestoneResponse {
            block_id: latest_milestone.milestone.block_id().to_string(),
            index: *latest_milestone.index,
        }),
    )
}
