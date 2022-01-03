// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::rejection::CustomRejection;

use bee_gossip::PeerId;
use bee_message::{
    address::{Address, Ed25519Address},
    milestone::MilestoneIndex,
    output::OutputId,
    payload::transaction::TransactionId,
    MessageId,
};

use warp::{reject, Filter, Rejection};

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

pub(super) fn bech32_address() -> impl Filter<Extract = (Address,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        Address::try_from_bech32(&value)
            .map_err(|_| reject::custom(CustomRejection::BadRequest("invalid address".to_string())))
    })
}

pub(super) fn ed25519_address() -> impl Filter<Extract = (Ed25519Address,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        value
            .parse::<Ed25519Address>()
            .map_err(|_| reject::custom(CustomRejection::BadRequest("invalid Ed25519 address".to_string())))
    })
}

pub(super) fn peer_id() -> impl Filter<Extract = (PeerId,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        value
            .parse::<PeerId>()
            .map_err(|_| reject::custom(CustomRejection::BadRequest("invalid peer id".to_string())))
    })
}
