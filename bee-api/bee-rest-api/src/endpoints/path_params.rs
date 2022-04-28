// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_gossip::PeerId;
use bee_message::{
    output::OutputId,
    payload::{
        milestone::{MilestoneId, MilestoneIndex},
        transaction::TransactionId,
    },
    MessageId,
};
use warp::{reject, Filter, Rejection};

use crate::endpoints::rejection::CustomRejection;

pub(super) fn output_id() -> impl Filter<Extract = (OutputId,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        value
            .parse::<OutputId>()
            .map_err(|_| reject::custom(CustomRejection::BadRequest("invalid output id".to_string())))
    })
}

pub(super) fn message_id() -> impl Filter<Extract = (MessageId,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        value
            .parse::<MessageId>()
            .map_err(|_| reject::custom(CustomRejection::BadRequest("invalid message id".to_string())))
    })
}

pub(super) fn transaction_id() -> impl Filter<Extract = (TransactionId,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        value
            .parse::<TransactionId>()
            .map_err(|_| reject::custom(CustomRejection::BadRequest("invalid transaction id".to_string())))
    })
}

pub(super) fn milestone_index() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        value
            .parse::<u32>()
            .map_err(|_| reject::custom(CustomRejection::BadRequest("invalid milestone index".to_string())))
            .map(MilestoneIndex)
    })
}

pub(super) fn milestone_id() -> impl Filter<Extract = (MilestoneId,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        value
            .parse::<MilestoneId>()
            .map_err(|_| reject::custom(CustomRejection::BadRequest("invalid milestone id".to_string())))
    })
}

pub(super) fn peer_id() -> impl Filter<Extract = (PeerId,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        value
            .parse::<PeerId>()
            .map_err(|_| reject::custom(CustomRejection::BadRequest("invalid peer id".to_string())))
    })
}
