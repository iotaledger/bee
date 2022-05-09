// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::MilestoneIndex;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        filters::with_args, path_params::milestone_index, rejection::CustomRejection, storage::StorageBackend,
        ApiArgsFullNode,
    },
    types::{body::SuccessBody, responses::MilestoneResponse},
};

fn path() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("milestones"))
        .and(milestone_index())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|milestone_index, args| async move { milestone(milestone_index, args) })
        .boxed()
}

pub(crate) fn milestone<B: StorageBackend>(
    milestone_index: MilestoneIndex,
    args: ApiArgsFullNode<B>,
) -> Result<impl Reply, Rejection> {
    match args.tangle.get_milestone(milestone_index) {
        Some(milestone) => Ok(warp::reply::json(&SuccessBody::new(MilestoneResponse {
            milestone_index: *milestone_index,
            message_id: milestone.message_id().to_string(),
            timestamp: milestone.timestamp(),
        }))),
        None => Err(reject::custom(CustomRejection::NotFound(
            "cannot find milestone".to_string(),
        ))),
    }
}
