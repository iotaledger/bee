// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_protocol::event::BlockSolidified;
use serde::Serialize;

use crate::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SolidInfoResponse {
    id: String,
}

impl From<BlockSolidified> for WsEvent {
    fn from(event: BlockSolidified) -> Self {
        Self::new(
            WsTopic::SolidInfo,
            WsEventInner::SolidInfo(SolidInfoResponse {
                id: event.block_id.to_string(),
            }),
        )
    }
}
