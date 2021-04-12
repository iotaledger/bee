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

impl From<TipAdded> for WsEvent {
    fn from(event: TipAdded) -> Self {
        Self::new(
            WsTopic::TipInfo,
            WsEventInner::TipInfo(TipInfoResponse {
                id: event.0.to_string(),
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
                id: event.0.to_string(),
                is_tip: false,
            }),
        )
    }
}
