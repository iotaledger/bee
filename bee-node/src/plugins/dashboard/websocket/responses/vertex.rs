// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

use bee_protocol::event::NewVertex;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct VertexResponse {
    id: String,
    parents: Vec<String>,
    is_solid: bool,
    is_referenced: bool,
    is_conflicting: bool,
    is_milestone: bool,
    is_tip: bool,
    is_selected: bool,
}

pub(crate) fn forward(message: NewVertex) -> WsEvent {
    message.into()
}

impl From<NewVertex> for WsEvent {
    fn from(val: NewVertex) -> Self {
        Self::new(WsTopic::Vertex, WsEventInner::Vertex(val.into()))
    }
}

impl From<NewVertex> for VertexResponse {
    fn from(val: NewVertex) -> Self {
        Self {
            id: val.id,
            parents: val.parent_ids,
            is_solid: val.is_solid,
            is_referenced: val.is_referenced,
            is_conflicting: val.is_conflicting,
            is_milestone: val.is_milestone,
            is_tip: val.is_tip,
            is_selected: val.is_selected,
        }
    }
}
