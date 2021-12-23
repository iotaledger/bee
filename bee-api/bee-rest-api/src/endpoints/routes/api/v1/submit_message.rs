// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::{RestApiConfig, ROUTE_SUBMIT_MESSAGE, ROUTE_SUBMIT_MESSAGE_RAW},
        filters::{with_message_submitter, with_network_id, with_protocol_config, with_rest_api_config, with_tangle},
        permission::has_permission,
        rejection::CustomRejection,
        storage::StorageBackend,
        NetworkId,
    },
    types::{body::SuccessBody, dtos::PayloadDto, responses::SubmitMessageResponse},
};

use bee_common::packable::Packable;
use bee_message::{parents::Parents, payload::Payload, Message, MessageBuilder, MessageId};
use bee_pow::providers::{miner::MinerBuilder, NonceProviderBuilder};
use bee_protocol::workers::{config::ProtocolConfig, MessageSubmitterError, MessageSubmitterWorkerEvent};
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;

use futures::channel::oneshot;
use log::error;
use serde_json::Value as JsonValue;
use tokio::sync::mpsc;
use warp::{filters::BoxedFilter, http::StatusCode, reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("messages")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::post())
        .and(
            (warp::header::exact("content-type", "application/json")
                .and(has_permission(
                    ROUTE_SUBMIT_MESSAGE,
                    public_routes.clone(),
                    allowed_ips.clone(),
                ))
                .and(warp::body::json())
                .and(with_tangle(tangle.clone()))
                .and(with_message_submitter(message_submitter.clone()))
                .and(with_network_id(network_id))
                .and(with_rest_api_config(rest_api_config))
                .and(with_protocol_config(protocol_config))
                .and_then(submit_message))
            .or(warp::header::exact("content-type", "application/octet-stream")
                .and(has_permission(ROUTE_SUBMIT_MESSAGE_RAW, public_routes, allowed_ips))
                .and(warp::body::bytes())
                .and(with_tangle(tangle))
                .and(with_message_submitter(message_submitter))
                .and_then(submit_message_raw)),
        )
        .boxed()
}

pub(crate) async fn submit_message<B: StorageBackend>(
    value: JsonValue,
    tangle: ResourceHandle<Tangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> Result<impl Reply, Rejection> {
    let network_id_v = &value["networkId"];
    let parents_v = &value["parentMessageIds"];
    let payload_v = &value["payload"];
    let nonce_v = &value["nonce"];

    // parse the values, take care of missing fields, build the message and submit it to the node for further
    // processing

    let network_id = if network_id_v.is_null() {
        network_id.1
    } else {
        network_id_v
            .as_str()
            .ok_or_else(|| {
                reject::custom(CustomRejection::BadRequest(
                    "invalid network id: expected an u64-string".to_string(),
                ))
            })?
            .parse::<u64>()
            .map_err(|_| {
                reject::custom(CustomRejection::BadRequest(
                    "invalid network id: expected an u64-string".to_string(),
                ))
            })?
    };

    let parents: Vec<MessageId> = if parents_v.is_null() {
        tangle.get_messages_to_approve().await.ok_or_else(|| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not auto-fill parents: no tips available".to_string(),
            ))
        })?
    } else {
        let array = parents_v.as_array().ok_or_else(|| {
            reject::custom(CustomRejection::BadRequest(
                "invalid parents: expected an array of message ids".to_string(),
            ))
        })?;
        let mut message_ids = Vec::with_capacity(array.len());
        for s in array {
            let message_id = s
                .as_str()
                .ok_or_else(|| {
                    reject::custom(CustomRejection::BadRequest(
                        "invalid parents: expected an array of message ids".to_string(),
                    ))
                })?
                .parse::<MessageId>()
                .map_err(|_| {
                    reject::custom(CustomRejection::BadRequest(
                        "invalid network id: expected an u64-string".to_string(),
                    ))
                })?;
            message_ids.push(message_id);
        }
        message_ids
    };

    let payload = if payload_v.is_null() {
        None
    } else {
        let payload_dto = serde_json::from_value::<PayloadDto>(payload_v.clone())
            .map_err(|e| reject::custom(CustomRejection::BadRequest(e.to_string())))?;
        Some(Payload::try_from(&payload_dto).map_err(|e| reject::custom(CustomRejection::BadRequest(e.to_string())))?)
    };

    let nonce = if nonce_v.is_null() {
        None
    } else {
        let parsed = nonce_v
            .as_str()
            .ok_or_else(|| {
                reject::custom(CustomRejection::BadRequest(
                    "invalid nonce: expected an u64-string".to_string(),
                ))
            })?
            .parse::<u64>()
            .map_err(|_| {
                reject::custom(CustomRejection::BadRequest(
                    "invalid nonce: expected an u64-string".to_string(),
                ))
            })?;
        if parsed == 0 { None } else { Some(parsed) }
    };

    let message = build_message(network_id, parents, payload, nonce, rest_api_config, protocol_config).await?;
    let message_id = forward_to_message_submitter(message, tangle, message_submitter).await?;

    Ok(warp::reply::with_status(
        warp::reply::json(&SuccessBody::new(SubmitMessageResponse {
            message_id: message_id.to_string(),
        })),
        StatusCode::CREATED,
    ))
}

pub(crate) async fn build_message(
    network_id: u64,
    parents: Vec<MessageId>,
    payload: Option<Payload>,
    nonce: Option<u64>,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> Result<Message, Rejection> {
    let message = if let Some(nonce) = nonce {
        let mut builder = MessageBuilder::new()
            .with_network_id(network_id)
            .with_parents(
                Parents::new(parents).map_err(|e| reject::custom(CustomRejection::BadRequest(e.to_string())))?,
            )
            .with_nonce_provider(nonce, 0f64);
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder
            .finish()
            .map_err(|e| reject::custom(CustomRejection::BadRequest(e.to_string())))?
    } else {
        if !rest_api_config.feature_proof_of_work() {
            return Err(reject::custom(CustomRejection::ServiceUnavailable(
                "can not auto-fill nonce: feature `PoW` not enabled".to_string(),
            )));
        }
        let mut builder = MessageBuilder::new()
            .with_network_id(network_id)
            .with_parents(
                Parents::new(parents).map_err(|e| reject::custom(CustomRejection::BadRequest(e.to_string())))?,
            )
            .with_nonce_provider(
                MinerBuilder::new().with_num_workers(num_cpus::get()).finish(),
                protocol_config.minimum_pow_score(),
            );
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder
            .finish()
            .map_err(|e| reject::custom(CustomRejection::BadRequest(e.to_string())))?
    };
    Ok(message)
}

pub(crate) async fn submit_message_raw<B: StorageBackend>(
    buf: warp::hyper::body::Bytes,
    tangle: ResourceHandle<Tangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> Result<impl Reply, Rejection> {
    let message = Message::unpack(&mut &(*buf)).map_err(|e| {
        reject::custom(CustomRejection::BadRequest(format!(
            "can not submit message: invalid bytes provided: the message format is not respected: {}",
            e
        )))
    })?;
    let message_id = forward_to_message_submitter(message, tangle, message_submitter).await?;
    Ok(warp::reply::with_status(
        warp::reply::json(&SuccessBody::new(SubmitMessageResponse {
            message_id: message_id.to_string(),
        })),
        StatusCode::CREATED,
    ))
}

pub(crate) async fn forward_to_message_submitter<B: StorageBackend>(
    message: Message,
    tangle: ResourceHandle<Tangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> Result<MessageId, Rejection> {
    let (message_id, message_bytes) = message.id();

    if tangle.contains(&message_id).await {
        return Ok(message_id);
    }

    let (notifier, waiter) = oneshot::channel::<Result<MessageId, MessageSubmitterError>>();

    message_submitter
        .send(MessageSubmitterWorkerEvent {
            message: message_bytes,
            notifier,
        })
        .map_err(|e| {
            error!("can not submit message: {}", e);
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not submit message".to_string(),
            ))
        })?;

    match waiter.await.map_err(|e| {
        error!("can not submit message: {}", e);
        reject::custom(CustomRejection::ServiceUnavailable(
            "can not submit message".to_string(),
        ))
    })? {
        Ok(message_id) => Ok(message_id),
        Err(e) => Err(reject::custom(CustomRejection::BadRequest(format!(
            "can not submit message: message is invalid: {}",
            e
        )))),
    }
}
