// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_MESSAGES_FIND, filters::with_storage, permission::has_permission, rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::{body::SuccessBody, responses::MessagesFindResponse},
};

use bee_message::{
    payload::indexation::{IndexationPayload, PaddedIndex},
    MessageId,
};
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::{collections::HashMap, net::IpAddr};

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("messages")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    storage: ResourceHandle<B>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGES_FIND, public_routes, allowed_ips))
        .and(warp::query().and_then(|query: HashMap<String, String>| async move {
            match query.get("index") {
                Some(i) => Ok(i.to_string()),
                None => Err(reject::custom(CustomRejection::BadRequest(
                    "invalid query parameter".to_string(),
                ))),
            }
        }))
        .and(with_storage(storage))
        .and_then(|index, storage| async move { messages_find(index, storage) })
        .boxed()
}

pub(crate) fn messages_find<B: StorageBackend>(
    index: String,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let index_bytes = hex::decode(index.clone())
        .map_err(|_| reject::custom(CustomRejection::BadRequest("Invalid index".to_owned())))?;
    let hashed_index = IndexationPayload::new(&index_bytes, &[]).unwrap().padded_index();

    let mut fetched = match Fetch::<PaddedIndex, Vec<MessageId>>::fetch(&*storage, &hashed_index).map_err(|_| {
        reject::custom(CustomRejection::ServiceUnavailable(
            "can not fetch from storage".to_string(),
        ))
    })? {
        Some(ids) => ids,
        None => vec![],
    };

    let count = fetched.len();
    let max_results = 1000;
    fetched.truncate(max_results);

    Ok(warp::reply::json(&SuccessBody::new(MessagesFindResponse {
        index,
        max_results,
        count,
        message_ids: fetched.iter().map(|id| id.to_string()).collect(),
    })))
}
