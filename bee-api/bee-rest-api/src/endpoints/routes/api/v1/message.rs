// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        filters::with_args, path_params::message_id, rejection::CustomRejection, storage::StorageBackend, ApiArgs,
    },
    types::{body::SuccessBody, dtos::MessageDto, responses::MessageResponse},
};

use bee_message::MessageId;

use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::sync::Arc;

fn path() -> impl Filter<Extract = (MessageId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("messages"))
        .and(message_id())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: Arc<ApiArgs<B>>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(message)
        .boxed()
}

pub(crate) async fn message<B: StorageBackend>(
    message_id: MessageId,
    args: Arc<ApiArgs<B>>,
) -> Result<impl Reply, Rejection> {
    match args.tangle.get(&message_id).await.map(|m| (*m).clone()) {
        Some(message) => Ok(warp::reply::json(&SuccessBody::new(MessageResponse(MessageDto::from(
            &message,
        ))))),
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find message".to_string(),
        ))),
    }
}
