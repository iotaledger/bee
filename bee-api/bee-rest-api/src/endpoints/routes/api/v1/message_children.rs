// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::MessageId;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        filters::with_args, path_params::message_id, routes::api::v1::MAX_RESPONSE_RESULTS, storage::StorageBackend,
        ApiArgsFullNode,
    },
    types::{body::SuccessBody, responses::MessageChildrenResponse},
};

fn path() -> impl Filter<Extract = (MessageId,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("messages"))
        .and(message_id())
        .and(warp::path("children"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|message_id, args| async move { message_children(message_id, args) })
        .boxed()
}

pub(crate) fn message_children<B: StorageBackend>(
    message_id: MessageId,
    args: ApiArgsFullNode<B>,
) -> Result<impl Reply, Rejection> {
    let mut children = Vec::from_iter(args.tangle.get_children(&message_id).unwrap_or_default());
    let count = children.len();
    children.truncate(MAX_RESPONSE_RESULTS);
    Ok(warp::reply::json(&SuccessBody::new(MessageChildrenResponse {
        message_id: message_id.to_string(),
        max_results: MAX_RESPONSE_RESULTS,
        count,
        children_message_ids: children.iter().map(|id| id.to_string()).collect(),
    })))
}
