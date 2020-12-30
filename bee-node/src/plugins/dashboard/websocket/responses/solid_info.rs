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
use bee_protocol::event::MessageSolidified;

use futures::executor::block_on;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SolidInfoResponse {
    id: String,
}

pub(crate) fn register(bus: ResHandle<Bus>, users: WsUsers) {
    bus.add_listener::<(), MessageSolidified, _>(move |message_solidified: &MessageSolidified| {
        let event = WsEvent::new(
            WsTopic::SolidInfo,
            WsEventInner::SolidInfo(SolidInfoResponse {
                id: message_solidified.0.to_string(),
            }),
        );
        block_on(broadcast(event, users.clone()))
    });
}
