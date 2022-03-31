// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_message::{MessageDto, MessageId};
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::ROUTE_MESSAGE, filters::with_tangle, path_params::message_id, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend,
    },
    types::responses::MessageResponse,
};

fn path() -> impl Filter<Extract = (MessageId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("messages"))
        .and(message_id())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGE, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(message)
        .boxed()
}

pub(crate) async fn message<B: StorageBackend>(
    message_id: MessageId,
    tangle: ResourceHandle<Tangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&message_id).await.map(|m| (*m).clone()) {
        Some(message) => Ok(warp::reply::json(&MessageResponse(MessageDto::from(&message)))),
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find message".to_string(),
        ))),
    }
}
