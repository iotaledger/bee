// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::rejection::CustomRejection;

use bee_message::{
    address::{Address, Ed25519Address},
    milestone::MilestoneIndex,
    output::OutputId,
    MessageId,
};
use bee_network::PeerId;

use warp::{reject, Filter, Rejection};

pub(super) fn output_id() -> impl Filter<Extract = (OutputId,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        match value.parse::<OutputId>() {
            Ok(id) => Ok(id),
            Err(_) => Err(reject::custom(CustomRejection::BadRequest(
                "invalid output id".to_string(),
            ))),
        }
    })
}

pub(super) fn message_id() -> impl Filter<Extract = (MessageId,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        match value.parse::<MessageId>() {
            Ok(msg) => Ok(msg),
            Err(_) => Err(reject::custom(CustomRejection::BadRequest(
                "invalid message id".to_string(),
            ))),
        }
    })
}

pub(super) fn milestone_index() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        match value.parse::<u32>() {
            Ok(i) => Ok(MilestoneIndex(i)),
            Err(_) => Err(reject::custom(CustomRejection::BadRequest(
                "invalid milestone index".to_string(),
            ))),
        }
    })
}

pub(super) fn bech32_address() -> impl Filter<Extract = (Address,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        match Address::try_from_bech32(&value) {
            Ok(addr) => Ok(addr),
            Err(_) => Err(reject::custom(CustomRejection::BadRequest(
                "invalid address".to_string(),
            ))),
        }
    })
}

pub(super) fn ed25519_address() -> impl Filter<Extract = (Ed25519Address,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        match value.parse::<Ed25519Address>() {
            Ok(addr) => Ok(addr),
            Err(_) => Err(reject::custom(CustomRejection::BadRequest(
                "invalid Ed25519 address".to_string(),
            ))),
        }
    })
}

pub(super) fn peer_id() -> impl Filter<Extract = (PeerId,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        match value.parse::<PeerId>() {
            Ok(id) => Ok(id),
            Err(_) => Err(reject::custom(CustomRejection::BadRequest(
                "invalid peer id".to_string(),
            ))),
        }
    })
}
