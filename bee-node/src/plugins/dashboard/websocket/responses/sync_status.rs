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
use bee_protocol::{event::LatestMilestoneChanged, tangle::MsTangle};
use bee_storage::storage::Backend;

use futures::executor::block_on;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SyncStatusResponse {
    lmi: u32,
    lsmi: u32, // Shouldn't it be smi (solid milestone index) instead?
}

pub(crate) fn register<B: Backend>(bus: ResHandle<Bus>, users: WsUsers, tangle: ResHandle<MsTangle<B>>) {
    bus.add_listener::<(), LatestMilestoneChanged, _>(move |latest_milestone: &LatestMilestoneChanged| {
        let event = WsEvent::new(
            WsTopic::SyncStatus,
            WsEventInner::SyncStatus(SyncStatusResponse {
                lmi: *latest_milestone.index,
                lsmi: *tangle.get_latest_solid_milestone_index(),
            }),
        );
        block_on(broadcast(event, users.clone()))
    });
}
