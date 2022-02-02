// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        filters::with_args, path_params::milestone_index, rejection::CustomRejection, storage::StorageBackend, ApiArgs,
    },
    types::{body::SuccessBody, responses::MilestoneResponse},
};

use bee_message::milestone::MilestoneIndex;

use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::sync::Arc;

fn path() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("milestones"))
        .and(milestone_index())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: Arc<ApiArgs<B>>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(milestone)
        .boxed()
}

pub(crate) async fn milestone<B: StorageBackend>(
    milestone_index: MilestoneIndex,
    args: Arc<ApiArgs<B>>,
) -> Result<impl Reply, Rejection> {
    match args.tangle.get_milestone_message_id(milestone_index).await {
        Some(message_id) => match args.tangle.get_metadata(&message_id).await {
            Some(metadata) => Ok(warp::reply::json(&SuccessBody::new(MilestoneResponse {
                milestone_index: *milestone_index,
                message_id: message_id.to_string(),
                timestamp: metadata.arrival_timestamp(),
            }))),
            None => Err(reject::custom(CustomRejection::NotFound(
                "can not find metadata for milestone".to_string(),
            ))),
        },
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find milestone".to_string(),
        ))),
    }
}
