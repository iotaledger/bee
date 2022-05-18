// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_message::{
    payload::indexation::{IndexationPayload, PaddedIndex},
    MessageId,
};
use bee_storage::backend::StorageBackendExt;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        filters::with_args, rejection::CustomRejection, routes::api::v1::MAX_RESPONSE_RESULTS, storage::StorageBackend,
        ApiArgsFullNode,
    },
    types::{body::SuccessBody, responses::MessagesFindResponse},
};

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("messages")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(warp::query().and_then(|query: HashMap<String, String>| async move {
            match query.get("index") {
                Some(i) => Ok(i.to_string()),
                None => Err(reject::custom(CustomRejection::BadRequest(
                    "invalid query parameter".to_string(),
                ))),
            }
        }))
        .and(with_args(args))
        .and_then(|index, args| async move { messages_find(index, args) })
        .boxed()
}

pub(crate) fn messages_find<B: StorageBackend>(
    index: String,
    args: ApiArgsFullNode<B>,
) -> Result<impl Reply, Rejection> {
    let index_bytes = hex::decode(index.clone())
        .map_err(|_| reject::custom(CustomRejection::BadRequest("Invalid index".to_owned())))?;
    let hashed_index = IndexationPayload::new(&index_bytes, &[]).unwrap().padded_index();

    let all_message_ids = match args.storage.fetch::<PaddedIndex, Vec<MessageId>>(&hashed_index) {
        Ok(result) => match result {
            Some(ids) => ids,
            None => vec![],
        },
        Err(_) => {
            return Err(reject::custom(CustomRejection::ServiceUnavailable(
                "can not fetch from storage".to_string(),
            )));
        }
    };

    let truncated_message_ids = all_message_ids
        .iter()
        .take(MAX_RESPONSE_RESULTS)
        .map(MessageId::to_string)
        .collect::<Vec<String>>();

    Ok(warp::reply::json(&SuccessBody::new(MessagesFindResponse {
        index,
        max_results: MAX_RESPONSE_RESULTS,
        count: truncated_message_ids.len(),
        message_ids: truncated_message_ids,
    })))
}
