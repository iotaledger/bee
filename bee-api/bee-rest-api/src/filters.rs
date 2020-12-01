// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{handlers, NetworkId};
use bee_common::node::ResHandle;
use serde::de::DeserializeOwned;
use warp::{reject, Filter, Rejection};

use bee_protocol::{tangle::MsTangle, MessageSubmitterWorkerEvent};

use crate::{filters::CustomRejection::BadRequest, storage::Backend};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) enum CustomRejection {
    BadRequest(&'static str),
    NotFound(&'static str),
    ServiceUnavailable(&'static str),
}

impl reject::Reject for CustomRejection {}

pub fn all<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
    storage: ResHandle<B>,
    message_submitter: flume::Sender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_health(tangle.clone())
        .or(get_info(tangle.clone(), network_id.clone()).or(get_milestone_by_milestone_index(tangle.clone())))
        .or(get_tips(tangle.clone()))
        .or(post_json_message(
            message_submitter.clone(),
            network_id.clone(),
            tangle.clone(),
        ))
        .or(post_raw_message(message_submitter.clone()))
        .or(get_message_by_index(storage.clone()))
        .or(get_message_by_message_id(tangle.clone()))
        .or(get_message_metadata(tangle.clone()))
        .or(get_raw_message(tangle.clone()))
        .or(get_children_by_message_id(tangle.clone()))
        .or(get_output_by_output_id(storage.clone()))
        .or(get_outputs_for_address(storage.clone()))
        .or(get_balance_for_address(storage.clone()))
}

fn get_health<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("health"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_health)
}

fn get_info<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
    network_id: NetworkId,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("info"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and(with_network_id(network_id))
        .and_then(handlers::get_info)
}

fn get_tips<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("tips"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_tips)
}

fn post_json_message<B: Backend>(
    message_submitter: flume::Sender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(json_body())
        .and(with_message_submitter(message_submitter))
        .and(with_network_id(network_id))
        .and(with_tangle(tangle))
        .and_then(handlers::post_json_message)
}

fn post_raw_message(
    message_submitter: flume::Sender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::body::bytes())
        .and(with_message_submitter(message_submitter))
        .and_then(handlers::post_raw_message)
}

fn get_message_by_index<B: Backend>(
    storage: ResHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::query().and_then(|query: HashMap<String, String>| async move {
            match query.get("index") {
                Some(i) => Ok(i.to_string()),
                None => Err(reject::custom(BadRequest("invalid query parameter"))),
            }
        }))
        .and(with_storage(storage))
        .and_then(handlers::get_message_by_index)
}

fn get_message_by_message_id<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_message_by_message_id)
}

fn get_message_metadata<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path("metadata"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_message_metadata)
}

fn get_raw_message<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path("raw"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_raw_message)
}

fn get_children_by_message_id<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path("children"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_children_by_message_id)
}

fn get_milestone_by_milestone_index<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("milestones"))
        .and(custom_path_param::milestone_index())
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_milestone_by_milestone_index)
}

fn get_output_by_output_id<B: Backend>(
    storage: ResHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("outputs"))
        .and(custom_path_param::output_id())
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::get_output_by_output_id)
}

fn get_balance_for_address<B: Backend>(
    storage: ResHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(custom_path_param::ed25519_address())
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::get_balance_for_address)
}

fn get_outputs_for_address<B: Backend>(
    storage: ResHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(custom_path_param::ed25519_address())
        .and(warp::path("outputs"))
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::get_outputs_for_address)
}

mod custom_path_param {

    use super::*;
    use bee_message::{payload::transaction::OutputId, prelude::Ed25519Address, MessageId};
    use bee_protocol::MilestoneIndex;

    pub(super) fn output_id() -> impl Filter<Extract = (OutputId,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<OutputId>() {
                Ok(id) => Ok(id),
                Err(_) => Err(reject::custom(BadRequest("invalid output id"))),
            }
        })
    }

    pub(super) fn message_id() -> impl Filter<Extract = (MessageId,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<MessageId>() {
                Ok(msg) => Ok(msg),
                Err(_) => Err(reject::custom(BadRequest("invalid message id"))),
            }
        })
    }

    pub(super) fn milestone_index() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<u32>() {
                Ok(i) => Ok(MilestoneIndex(i)),
                Err(_) => Err(reject::custom(BadRequest("invalid milestone index"))),
            }
        })
    }

    pub(super) fn ed25519_address() -> impl Filter<Extract = (Ed25519Address,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<Ed25519Address>() {
                Ok(addr) => Ok(addr),
                Err(_) => Err(reject::custom(BadRequest("invalid ed25519 address"))),
            }
        })
    }
}

fn with_network_id(
    network_id: NetworkId,
) -> impl Filter<Extract = (NetworkId,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || network_id.clone())
}

fn with_tangle<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = (ResHandle<MsTangle<B>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tangle.clone())
}

fn with_storage<B: Backend>(
    storage: ResHandle<B>,
) -> impl Filter<Extract = (ResHandle<B>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

fn with_message_submitter(
    message_submitter: flume::Sender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = (flume::Sender<MessageSubmitterWorkerEvent>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || message_submitter.clone())
}

fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}
