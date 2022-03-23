// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_protocol::workers::event::{TipAdded, TipRemoved};
use serde::Serialize;

use crate::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TipInfoResponse {
    id: String,
    is_tip: bool,
}

impl From<TipAdded> for WsEvent {
    fn from(event: TipAdded) -> Self {
        Self::new(
            WsTopic::TipInfo,
            WsEventInner::TipInfo(TipInfoResponse {
                id: event.message_id.to_string(),
                is_tip: true,
            }),
        )
    }
}

impl From<TipRemoved> for WsEvent {
    fn from(event: TipRemoved) -> Self {
        Self::new(
            WsTopic::TipInfo,
            WsEventInner::TipInfo(TipInfoResponse {
                id: event.message_id.to_string(),
                is_tip: false,
            }),
        )
    }
}
