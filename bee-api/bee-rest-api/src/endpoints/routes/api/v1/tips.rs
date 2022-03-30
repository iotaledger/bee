// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        filters::with_args, rejection::CustomRejection, storage::StorageBackend, ApiArgsFullNode, CONFIRMED_THRESHOLD,
    },
    types::{body::SuccessBody, responses::TipsResponse},
};

fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("tips")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(tips)
        .boxed()
}

pub(crate) async fn tips<B: StorageBackend>(args: ApiArgsFullNode<B>) -> Result<impl Reply, Rejection> {
    if !args.tangle.is_confirmed_threshold(CONFIRMED_THRESHOLD) {
        return Err(reject::custom(CustomRejection::ServiceUnavailable(
            "the node is not synchronized".to_string(),
        )));
    }
    match args.tangle.get_messages_to_approve().await {
        Some(tips) => Ok(warp::reply::json(&SuccessBody::new(TipsResponse {
            tip_message_ids: tips.iter().map(|t| t.to_string()).collect(),
        }))),
        None => Err(reject::custom(CustomRejection::ServiceUnavailable(
            "tip pool is empty".to_string(),
        ))),
    }
}
