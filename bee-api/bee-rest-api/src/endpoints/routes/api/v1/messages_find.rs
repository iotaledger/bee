// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        filters::with_args, rejection::CustomRejection, routes::api::v1::MAX_RESPONSE_RESULTS, storage::StorageBackend,
        ApiArgsFullNode,
    },
    types::{body::SuccessBody, responses::MessagesFindResponse},
};

use bee_message::{
    payload::indexation::{IndexationPayload, PaddedIndex},
    MessageId,
};
use bee_storage::access::Fetch;

use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::{collections::HashMap, sync::Arc};

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("messages")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: Arc<ApiArgsFullNode<B>>) -> BoxedFilter<(impl Reply,)> {
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
    args: Arc<ApiArgsFullNode<B>>,
) -> Result<impl Reply, Rejection> {
    let index_bytes = hex::decode(index.clone())
        .map_err(|_| reject::custom(CustomRejection::BadRequest("Invalid index".to_owned())))?;
    let hashed_index = IndexationPayload::new(&index_bytes, &[]).unwrap().padded_index();

    let mut fetched =
        match Fetch::<PaddedIndex, Vec<MessageId>>::fetch(&*args.storage, &hashed_index).map_err(|_| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not fetch from storage".to_string(),
            ))
        })? {
            Some(ids) => ids,
            None => vec![],
        };

    let count = fetched.len();
    fetched.truncate(MAX_RESPONSE_RESULTS);

    Ok(warp::reply::json(&SuccessBody::new(MessagesFindResponse {
        index,
        max_results: MAX_RESPONSE_RESULTS,
        count,
        message_ids: fetched.iter().map(|id| id.to_string()).collect(),
    })))
}
