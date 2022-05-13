// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_protocol::workers::event::VertexCreated;
use serde::Serialize;

use crate::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

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

impl From<VertexCreated> for WsEvent {
    fn from(event: VertexCreated) -> Self {
        Self::new(WsTopic::Vertex, WsEventInner::Vertex(event.into()))
    }
}

impl From<VertexCreated> for VertexResponse {
    fn from(event: VertexCreated) -> Self {
        Self {
            id: event.block_id.to_string(),
            parents: event.parent_block_ids.iter().map(|p| p.to_string()).collect(),
            is_solid: event.is_solid,
            is_referenced: event.is_referenced,
            is_conflicting: event.is_conflicting,
            is_milestone: event.is_milestone,
            is_tip: event.is_tip,
            is_selected: event.is_selected,
        }
    }
}
