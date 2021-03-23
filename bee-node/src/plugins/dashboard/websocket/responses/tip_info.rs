// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

use bee_protocol::workers::event::{TipAdded, TipRemoved};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TipInfoResponse {
    id: String,
    is_tip: bool,
}

pub(crate) fn forward_tip_added(event: TipAdded) -> WsEvent {
    WsEvent::new(
        WsTopic::TipInfo,
        WsEventInner::TipInfo(TipInfoResponse {
            id: event.0.to_string(),
            is_tip: true,
        }),
    )
}

pub(crate) fn forward_tip_removed(event: TipRemoved) -> WsEvent {
    WsEvent::new(
        WsTopic::TipInfo,
        WsEventInner::TipInfo(TipInfoResponse {
            id: event.0.to_string(),
            is_tip: false,
        }),
    )
}
