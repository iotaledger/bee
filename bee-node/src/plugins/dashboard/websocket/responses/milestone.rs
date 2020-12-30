// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::{
    broadcast,
    websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
        WsUsers,
    },
};

use bee_common::event::Bus;
use bee_common_pt2::node::ResHandle;
use bee_protocol::event::LatestMilestoneChanged;

use futures::executor::block_on;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct MilestoneResponse {
    #[serde(rename = "messageID")] // shouldn't it be messageId to be consistent with the REST API?
    message_id: String,
    index: u32,
}

pub(crate) fn register(bus: ResHandle<Bus>, users: WsUsers) {
    bus.add_listener::<(), LatestMilestoneChanged, _>(move |latest_milestone: &LatestMilestoneChanged| {
        let event = WsEvent::new(
            WsTopic::Milestone,
            WsEventInner::Milestone(MilestoneResponse {
                message_id: latest_milestone.milestone.message_id().to_string(),
                index: *latest_milestone.index,
            }),
        );
        block_on(broadcast(event, users.clone()))
    });
}
