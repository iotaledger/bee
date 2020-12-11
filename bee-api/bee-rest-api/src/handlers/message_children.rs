// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    handlers::{EnvelopeContent, SuccessEnvelope},
    storage::Backend,
};
use bee_common::node::ResHandle;
use bee_message::prelude::*;
use bee_protocol::tangle::MsTangle;
use serde::Serialize;
use std::iter::FromIterator;
use warp::{Rejection, Reply};

pub async fn message_children<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    let mut children = Vec::from_iter(tangle.get_children(&message_id));
    let count = children.len();
    let max_results = 1000;
    children.truncate(max_results);
    Ok(warp::reply::json(&SuccessEnvelope::new(GetChildrenResponse {
        message_id: message_id.to_string(),
        max_results,
        count,
        children_message_ids: children.iter().map(|id| id.to_string()).collect(),
    })))
}

/// Response of GET /api/v1/messages/{message_id}/children
#[derive(Clone, Debug, Serialize)]
pub struct GetChildrenResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "childrenMessageIds")]
    pub children_message_ids: Vec<String>,
}

impl EnvelopeContent for GetChildrenResponse {}
