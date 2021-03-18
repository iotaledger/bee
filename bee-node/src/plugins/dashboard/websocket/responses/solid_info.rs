// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

use bee_protocol::workers::event::MessageSolidified;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SolidInfoResponse {
    id: String,
}

pub(crate) fn forward(message_solidified: MessageSolidified) -> WsEvent {
    WsEvent::new(
        WsTopic::SolidInfo,
        WsEventInner::SolidInfo(SolidInfoResponse {
            id: message_solidified.0.to_string(),
        }),
    )
}
