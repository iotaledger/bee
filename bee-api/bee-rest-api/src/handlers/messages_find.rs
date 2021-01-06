// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::ServiceUnavailable,
    handlers::{BodyInner, SuccessBody},
    storage::StorageBackend,
};

use bee_common_pt2::node::ResHandle;
use bee_message::prelude::*;
use bee_storage::access::Fetch;

use serde::Serialize;
use warp::{reject, Rejection, Reply};

use std::ops::Deref;

pub(crate) async fn messages_find<B: StorageBackend>(
    index: String,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    let hashed_index = Indexation::new(index.clone(), &[]).unwrap().hash();

    let mut fetched = match Fetch::<HashedIndex, Vec<MessageId>>::fetch(storage.deref(), &hashed_index)
        .await
        .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?
    {
        Some(ids) => ids,
        None => vec![],
    };

    let count = fetched.len();
    let max_results = 1000;
    fetched.truncate(max_results);

    Ok(warp::reply::json(&SuccessBody::new(MessagesForIndexResponse {
        index,
        max_results,
        count,
        message_ids: fetched.iter().map(|id| id.to_string()).collect(),
    })))
}

/// Response of GET /api/v1/messages/{message_id}?index={INDEX}
#[derive(Clone, Debug, Serialize)]
pub struct MessagesForIndexResponse {
    pub index: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "messageIds")]
    pub message_ids: Vec<String>,
}

impl BodyInner for MessagesForIndexResponse {}
