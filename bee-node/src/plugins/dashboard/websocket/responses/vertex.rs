// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

use bee_protocol::workers::event::NewVertex;

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

impl From<NewVertex> for WsEvent {
    fn from(event: NewVertex) -> Self {
        Self::new(WsTopic::Vertex, WsEventInner::Vertex(event.into()))
    }
}

impl From<NewVertex> for VertexResponse {
    fn from(event: NewVertex) -> Self {
        Self {
            id: event.id,
            parents: event.parent_ids,
            is_solid: event.is_solid,
            is_referenced: event.is_referenced,
            is_conflicting: event.is_conflicting,
            is_milestone: event.is_milestone,
            is_tip: event.is_tip,
            is_selected: event.is_selected,
        }
    }
}
