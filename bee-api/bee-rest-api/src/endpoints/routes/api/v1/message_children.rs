// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_MESSAGE_CHILDREN, filters::with_tangle, path_params::message_id, permission::has_permission,
        storage::StorageBackend,
    },
    types::{body::SuccessBody, responses::MessageChildrenResponse},
};

use bee_message::MessageId;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use warp::{Filter, Rejection, Reply};

use std::{iter::FromIterator, net::IpAddr};

fn path() -> impl Filter<Extract = (MessageId,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("messages"))
        .and(message_id())
        .and(warp::path("children"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGE_CHILDREN, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(message_children)
}

pub async fn message_children<B: StorageBackend>(
    message_id: MessageId,
    tangle: ResourceHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    let mut children = Vec::from_iter(tangle.get_children(&message_id).await.unwrap_or_default());
    let count = children.len();
    let max_results = 1000;
    children.truncate(max_results);
    Ok(warp::reply::json(&SuccessBody::new(MessageChildrenResponse {
        message_id: message_id.to_string(),
        max_results,
        count,
        children_message_ids: children.iter().map(|id| id.to_string()).collect(),
    })))
}
