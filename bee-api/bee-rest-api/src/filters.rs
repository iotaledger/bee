// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{config::RestApiConfig, filters::CustomRejection::BadRequest, handlers, storage::Backend, NetworkId};

use bee_common::packable::Packable;
use bee_common_pt2::node::ResHandle;
use bee_protocol::{config::ProtocolConfig, tangle::MsTangle, MessageSubmitterWorkerEvent};

use bech32::FromBase32;
use tokio::sync::mpsc;
use warp::{reject, Filter, Rejection};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) enum CustomRejection {
    BadRequest(String),
    NotFound(String),
    ServiceUnavailable(String),
}

impl reject::Reject for CustomRejection {}

pub fn all<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
    storage: ResHandle<B>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    health(tangle.clone()).or(info(
        tangle.clone(),
        network_id.clone(),
        rest_api_config.clone(),
        protocol_config.clone(),
    )
    .or(tips(tangle.clone()))
    .or(submit_message(
        tangle.clone(),
        message_submitter.clone(),
        network_id,
        rest_api_config,
        protocol_config,
    ))
    .or(submit_message_raw(tangle.clone(), message_submitter))
    .or(message_indexation(storage.clone()))
    .or(message(tangle.clone()))
    .or(message_metadata(tangle.clone()))
    .or(message_raw(tangle.clone()))
    .or(message_children(tangle.clone()))
    .or(output(storage.clone()))
    .or(balance_bech32(storage.clone()))
    .or(balance_ed25519(storage.clone()))
    .or(outputs_bech32(storage.clone()))
    .or(outputs_ed25519(storage))
    .or(milestone(tangle)))
}

fn health<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("health"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::health::health)
}

fn info<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
    network_id: NetworkId,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("info"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and(with_network_id(network_id))
        .and(with_rest_api_config(rest_api_config))
        .and(with_protocol_config(protocol_config))
        .and_then(handlers::info::info)
}

fn tips<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("tips"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::tips::tips)
}

fn submit_message<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::body::json())
        .and(with_tangle(tangle))
        .and(with_message_submitter(message_submitter))
        .and(with_network_id(network_id))
        .and(with_rest_api_config(rest_api_config))
        .and(with_protocol_config(protocol_config))
        .and_then(handlers::submit_message::submit_message)
}

fn submit_message_raw<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::body::bytes())
        .and(with_tangle(tangle))
        .and(with_message_submitter(message_submitter))
        .and_then(handlers::submit_message_raw::submit_message_raw)
}

fn message_indexation<B: Backend>(
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
                None => Err(reject::custom(BadRequest("invalid query parameter".to_string()))),
            }
        }))
        .and(with_storage(storage))
        .and_then(handlers::messages_find::messages_find)
}

fn message<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::message::message)
}

fn message_metadata<B: Backend>(
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
        .and_then(handlers::message_metadata::message_metadata)
}

fn message_raw<B: Backend>(
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
        .and_then(handlers::message_raw::message_raw)
}

fn message_children<B: Backend>(
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
        .and_then(handlers::message_children::message_children)
}

fn output<B: Backend>(
    storage: ResHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("outputs"))
        .and(custom_path_param::output_id())
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::output::output)
}

fn balance_bech32<B: Backend>(
    storage: ResHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(custom_path_param::bech32_address())
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::balance_bech32::balance_bech32)
}

fn balance_ed25519<B: Backend>(
    storage: ResHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(warp::path("ed25519"))
        .and(custom_path_param::ed25519_address())
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::balance_ed25519::balance_ed25519)
}

fn outputs_bech32<B: Backend>(
    storage: ResHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(custom_path_param::bech32_address())
        .and(warp::path("outputs"))
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::outputs_bech32::outputs_bech32)
}

fn outputs_ed25519<B: Backend>(
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
        .and_then(handlers::outputs_ed25519::outputs_ed25519)
}

fn milestone<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("milestones"))
        .and(custom_path_param::milestone_index())
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::milestone::milestone)
}

mod custom_path_param {

    use super::*;
    use bee_message::{
        payload::transaction::{Address, OutputId},
        prelude::Ed25519Address,
        MessageId,
    };
    use bee_protocol::MilestoneIndex;

    pub(super) fn output_id() -> impl Filter<Extract = (OutputId,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<OutputId>() {
                Ok(id) => Ok(id),
                Err(_) => Err(reject::custom(BadRequest("invalid output id".to_string()))),
            }
        })
    }

    pub(super) fn message_id() -> impl Filter<Extract = (MessageId,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<MessageId>() {
                Ok(msg) => Ok(msg),
                Err(_) => Err(reject::custom(BadRequest("invalid message id".to_string()))),
            }
        })
    }

    pub(super) fn milestone_index() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<u32>() {
                Ok(i) => Ok(MilestoneIndex(i)),
                Err(_) => Err(reject::custom(BadRequest("invalid milestone index".to_string()))),
            }
        })
    }

    pub(super) fn bech32_address() -> impl Filter<Extract = (Address,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match bech32::decode(&value) {
                Ok((hrp, data)) => {
                    if hrp.eq("iot") || hrp.eq("toi") {
                        let bytes = Vec::<u8>::from_base32(&data)
                            .map_err(|_| reject::custom(BadRequest("invalid IOTA address".to_string())))?;
                        Ok(Address::unpack(&mut bytes.as_slice())
                            .map_err(|_| reject::custom(BadRequest("invalid IOTA address".to_string())))?)
                    } else {
                        Err(reject::custom(BadRequest("not an IOTA address".to_string())))
                    }
                }
                Err(_) => Err(reject::custom(BadRequest("not a bech32 address".to_string()))),
            }
        })
    }

    pub(super) fn ed25519_address() -> impl Filter<Extract = (Ed25519Address,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<Ed25519Address>() {
                Ok(addr) => Ok(addr),
                Err(_) => Err(reject::custom(BadRequest("invalid Ed25519 address".to_string()))),
            }
        })
    }
}

fn with_network_id(
    network_id: NetworkId,
) -> impl Filter<Extract = (NetworkId,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || network_id.clone())
}

fn with_rest_api_config(
    config: RestApiConfig,
) -> impl Filter<Extract = (RestApiConfig,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

fn with_protocol_config(
    config: ProtocolConfig,
) -> impl Filter<Extract = (ProtocolConfig,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
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
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = (mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || message_submitter.clone())
}
