// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::ROUTE_MESSAGE_RAW,
    filters::with_tangle,
    path_params::message_id,
    permission::has_permission,
    rejection::CustomRejection, 
    storage::StorageBackend
};

use bee_common::packable::Packable;
use bee_message::MessageId;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use warp::{Filter, http::Response, reject, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (MessageId,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("messages"))
        .and(message_id())
        .and(warp::path("raw"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()    
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGE_RAW, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(message_raw)
}

pub async fn message_raw<B: StorageBackend>(
    message_id: MessageId,
    tangle: ResourceHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&message_id).await.map(|m| (*m).clone()) {
        Some(message) => Ok(Response::builder()
            .header("Content-Type", "application/octet-stream")
            .body(message.pack_new())),
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find message".to_string(),
        ))),
    }
}
