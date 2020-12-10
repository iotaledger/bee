// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::ServiceUnavailable,
    handlers::{EnvelopeContent, SuccessEnvelope},
    storage::Backend,
};
use bee_common::node::ResHandle;
use bee_message::prelude::*;
use bee_storage::access::Fetch;
use blake2::Blake2s;
use serde::Serialize;
use std::{convert::TryInto, ops::Deref};
use warp::{reject, Rejection, Reply};

pub(crate) async fn message_indexation<B: Backend>(
    index: String,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    let hashed_index = {
        use digest::Digest;
        let mut hasher = Blake2s::new();
        hasher.update(index.as_bytes());
        // `Blake2s` output is `HASHED_INDEX_LENGTH` bytes long.
        HashedIndex::new(hasher.finalize_reset().as_slice().try_into().unwrap())
    };

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

    Ok(warp::reply::json(&SuccessEnvelope::new(GetMessagesByIndexResponse {
        index,
        max_results,
        count,
        message_ids: fetched.iter().map(|id| id.to_string()).collect(),
    })))
}

/// Response of GET /api/v1/messages/{message_id}?index={INDEX}
#[derive(Clone, Debug, Serialize)]
pub struct GetMessagesByIndexResponse {
    pub index: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "messageIds")]
    pub message_ids: Vec<String>,
}

impl EnvelopeContent for GetMessagesByIndexResponse {}
