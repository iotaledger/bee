// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    filters::with_args, path_params::message_id, rejection::CustomRejection, storage::StorageBackend, ApiArgsFullNode,
};

use bee_common::packable::Packable;
use bee_message::MessageId;

use warp::{filters::BoxedFilter, http::Response, reject, Filter, Rejection, Reply};

fn path() -> impl Filter<Extract = (MessageId,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("messages"))
        .and(message_id())
        .and(warp::path("raw"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|message_id, args| async move { message_raw(message_id, args) })
        .boxed()
}

pub(crate) fn message_raw<B: StorageBackend>(
    message_id: MessageId,
    args: ApiArgsFullNode<B>,
) -> Result<impl Reply, Rejection> {
    match args.tangle.get(&message_id) {
        Some(message) => Ok(Response::builder()
            .header("Content-Type", "application/octet-stream")
            .body(message.pack_new())),
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find message".to_string(),
        ))),
    }
}
