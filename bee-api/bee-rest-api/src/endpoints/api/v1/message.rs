// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    config::ROUTE_MESSAGE,
    filters::with_tangle,
    path_params::message_id,
    permission::has_permission,
    rejection::CustomRejection,
    storage::StorageBackend,
    types::MessageDto,
};

use bee_message::MessageId;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use serde::Serialize;
use warp::{Filter, reject, Rejection, Reply};

use std::{convert::TryFrom, net::IpAddr};

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(message_id())
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGE, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(message)
}

pub(crate) async fn message<B: StorageBackend>(
    message_id: MessageId,
    tangle: ResourceHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&message_id).await.map(|m| (*m).clone()) {
        Some(message) => Ok(warp::reply::json(&SuccessBody::new(MessageResponse(
            MessageDto::try_from(&message).map_err(|e| reject::custom(CustomRejection::BadRequest(e)))?,
        )))),
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find message".to_string(),
        ))),
    }
}

/// Response of GET /api/v1/messages/{message_id}
#[derive(Clone, Debug, Serialize)]
pub struct MessageResponse(pub MessageDto);

impl BodyInner for MessageResponse {}
