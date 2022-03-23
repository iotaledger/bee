// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        filters::with_args, path_params::message_id, rejection::CustomRejection, storage::StorageBackend,
        ApiArgsFullNode,
    },
    types::{body::SuccessBody, dtos::MessageDto, responses::MessageResponse},
};

use bee_message::MessageId;

use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

fn path() -> impl Filter<Extract = (MessageId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("messages"))
        .and(message_id())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|message_id, args| async move { message(message_id, tangle) })
        .boxed()
}

pub(crate) fn message<B: StorageBackend>(
    message_id: MessageId,
    args: ApiArgsFullNode<B>,
) -> Result<impl Reply, Rejection> {
    match args.tangle.get(&message_id) {
        Some(message) => Ok(warp::reply::json(&SuccessBody::new(MessageResponse(MessageDto::from(
            &message,
        ))))),
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find message".to_string(),
        ))),
    }
}
