// Copyright 2020-2021 IOTA Stiftung
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

impl From<MessageSolidified> for WsEvent {
    fn from(event: MessageSolidified) -> Self {
        Self::new(
            WsTopic::SolidInfo,
            WsEventInner::SolidInfo(SolidInfoResponse {
                id: event.message_id.to_string(),
            }),
        )
    }
}
