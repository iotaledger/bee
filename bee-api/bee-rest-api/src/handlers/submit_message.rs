// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::RestApiConfig,
    filters::CustomRejection::{BadRequest, ServiceUnavailable},
    handlers::{BodyInner, SuccessBody},
    storage::StorageBackend,
    types::*,
    NetworkId,
};

use bee_message::prelude::*;
use bee_pow::providers::{ConstantBuilder, MinerBuilder, ProviderBuilder};
use bee_protocol::{config::ProtocolConfig, MessageSubmitterError, MessageSubmitterWorkerEvent};
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use futures::channel::oneshot;
use log::error;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::convert::TryFrom;
use tokio::sync::mpsc;
use warp::{http::StatusCode, reject, Rejection, Reply};

pub(crate) async fn submit_message<B: StorageBackend>(
    value: JsonValue,
    tangle: ResourceHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> Result<impl Reply, Rejection> {
    let network_id_v = &value["network_id"];
    let parents_v = &value["parents"];
    let payload_v = &value["payload"];
    let nonce_v = &value["nonce"];

    // parse the values, take care of missing fields, build the message and submit it to the node for further
    // processing

    let network_id = if network_id_v.is_null() {
        network_id.1
    } else {
        network_id_v
            .as_str()
            .ok_or_else(|| reject::custom(BadRequest("invalid network id: expected an u64-string".to_string())))?
            .parse::<u64>()
            .map_err(|_| reject::custom(BadRequest("invalid network id: expected an u64-string".to_string())))?
    };

    let parents: Vec<MessageId> = if parents_v.is_null() {
        tangle.get_messages_to_approve().await.ok_or_else(|| {
            reject::custom(ServiceUnavailable(
                "can not auto-fill parents: no tips available".to_string(),
            ))
        })?
    } else {
        let array = parents_v.as_array().ok_or_else(|| {
            reject::custom(BadRequest(
                "invalid parents: expected an array of message ids".to_string(),
            ))
        })?;
        let mut message_ids = Vec::new();
        for s in array {
            let message_id = s
                .as_str()
                .ok_or_else(|| {
                    reject::custom(BadRequest(
                        "invalid parents: expected an array of message ids".to_string(),
                    ))
                })?
                .parse::<MessageId>()
                .map_err(|_| reject::custom(BadRequest("invalid network id: expected an u64-string".to_string())))?;
            message_ids.push(message_id);
        }
        message_ids
    };

    let payload = if payload_v.is_null() {
        None
    } else {
        let payload_dto = serde_json::from_value::<PayloadDto>(payload_v.clone())
            .map_err(|e| reject::custom(BadRequest(e.to_string())))?;
        Some(Payload::try_from(&payload_dto).map_err(|e| reject::custom(BadRequest(e)))?)
    };

    let nonce = if nonce_v.is_null() {
        None
    } else {
        let parsed = nonce_v
            .as_str()
            .ok_or_else(|| reject::custom(BadRequest("invalid nonce: expected an u64-string".to_string())))?
            .parse::<u64>()
            .map_err(|_| reject::custom(BadRequest("invalid nonce: expected an u64-string".to_string())))?;
        if parsed == 0 {
            None
        } else {
            Some(parsed)
        }
    };

    let message = if let Some(nonce) = nonce {
        let mut builder = MessageBuilder::new()
            .with_network_id(network_id)
            .with_parents(parents)
            .with_nonce_provider(ConstantBuilder::new().with_value(nonce).finish(), 0f64, None);
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder
            .finish()
            .map_err(|e| reject::custom(BadRequest(e.to_string())))?
    } else {
        if !rest_api_config.feature_proof_of_work() {
            return Err(reject::custom(ServiceUnavailable(
                "can not auto-fill nonce: feature `PoW` not enabled".to_string(),
            )));
        }
        let mut builder = MessageBuilder::new()
            .with_network_id(network_id)
            .with_parents(parents)
            .with_nonce_provider(
                MinerBuilder::new().with_num_workers(num_cpus::get()).finish(),
                protocol_config.minimum_pow_score(),
                None,
            );
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder
            .finish()
            .map_err(|e| reject::custom(BadRequest(e.to_string())))?
    };

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
    tangle: ResourceHandle<MsTangle<B>>,
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
            reject::custom(ServiceUnavailable("can not submit message".to_string()))
        })?;

    let result = waiter.await.map_err(|_| {
        // TODO: report back from HasherWorker and replace following line with:
        // error!("can not submit message: {:?}",e);
        reject::custom(BadRequest("invalid message recognized by hash-cache".to_string()))
    })?;

    match result {
        Ok(message_id) => Ok(message_id),
        Err(e) => Err(reject::custom(BadRequest(e.to_string()))),
    }
}

/// Response of POST /api/v1/messages
#[derive(Clone, Debug, Serialize)]
pub struct SubmitMessageResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
}

impl BodyInner for SubmitMessageResponse {}
