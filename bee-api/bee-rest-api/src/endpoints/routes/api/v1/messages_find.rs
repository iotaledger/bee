// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_MESSAGES_FIND, filters::with_storage, permission::has_permission, rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::{body::SuccessBody, responses::MessagesFindResponse},
};

use bee_message::{
    payload::indexation::{HashedIndex, IndexationPayload},
    MessageId,
};
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

use warp::{reject, Filter, Rejection, Reply};

use std::{collections::HashMap, net::IpAddr};

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("messages")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
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
        .and_then(messages_find)
}

pub(crate) async fn messages_find<B: StorageBackend>(
    index: String,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let index_bytes = hex::decode(index.clone())
        .map_err(|_| reject::custom(CustomRejection::BadRequest("Invalid index".to_owned())))?;
    let hashed_index = IndexationPayload::new(&index_bytes, &[]).unwrap().hash();

    let mut fetched = match Fetch::<HashedIndex, Vec<MessageId>>::fetch(&*storage, &hashed_index)
        .await
        .map_err(|_| {
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
