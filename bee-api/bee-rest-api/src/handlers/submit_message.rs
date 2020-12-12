// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::RestApiConfig,
    filters::CustomRejection::{BadRequest, ServiceUnavailable},
    handlers::{EnvelopeContent, SuccessEnvelope},
    storage::Backend,
    types::*,
    NetworkId,
};
use bee_common::{node::ResHandle, packable::Packable};
use bee_message::prelude::*;
use bee_pow::providers::{ConstantBuilder, MinerBuilder, ProviderBuilder};
use bee_protocol::{config::ProtocolConfig, tangle::MsTangle, MessageSubmitterError, MessageSubmitterWorkerEvent};
use blake2::VarBlake2b;
use futures::channel::oneshot;
use log::error;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::convert::TryFrom;
use tokio::sync::mpsc;
use warp::{http::StatusCode, reject, Rejection, Reply};

pub(crate) async fn submit_message<B: Backend>(
    value: JsonValue,
    tangle: ResHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> Result<impl Reply, Rejection> {
    let is_set = |value: &JsonValue| -> bool { !value.is_null() };

    // validate json input, take care of missing fields, try to build the message and submit it to the node for further
    // processing

    let network_id = {
        let network_id_v = &value["network_id"];
        if network_id_v.is_null() {
            network_id.1
        } else {
            network_id_v
                .as_str()
                .ok_or(reject::custom(BadRequest(
                    "invalid network id: expected an u64-string".to_string(),
                )))?
                .parse::<u64>()
                .map_err(|_| reject::custom(BadRequest("invalid network id: expected an u64-string".to_string())))?
        }
    };

    let (parent_1_message_id, parent_2_message_id) = {
        let parent_1_v = &value["parent1MessageId"];
        let parent_2_v = &value["parent2MessageId"];

        if is_set(parent_1_v) && is_set(parent_2_v) {
            // if both parents are set
            let parent1 = parent_1_v
                .as_str()
                .ok_or(reject::custom(BadRequest(format!(
                    "invalid parent 1: expected a hex-string of length {}",
                    MESSAGE_ID_LENGTH * 2
                ))))?
                .parse::<MessageId>()
                .map_err(|_| {
                    reject::custom(BadRequest(format!(
                        "invalid parent 1: expected a hex-string of length {}",
                        MESSAGE_ID_LENGTH * 2
                    )))
                })?;
            let parent2 = parent_2_v
                .as_str()
                .ok_or(reject::custom(BadRequest(format!(
                    "invalid parent 2: expected a hex-string of length {}",
                    MESSAGE_ID_LENGTH * 2
                ))))?
                .parse::<MessageId>()
                .map_err(|_| {
                    reject::custom(BadRequest(format!(
                        "invalid parent 2: expected a hex-string of length {}",
                        MESSAGE_ID_LENGTH * 2
                    )))
                })?;
            (parent1, parent2)
        } else if parent_1_v.is_null() && parent_2_v.is_null() {
            // if none of the parents are set
            tangle
                .get_messages_to_approve()
                .await
                .ok_or(reject::custom(ServiceUnavailable(
                    "can not auto-fill tips: tip pool is empty".to_string(),
                )))?
        } else {
            // if only one parent is set
            let parent = if is_set(parent_1_v) {
                parent_1_v
                    .as_str()
                    .ok_or(reject::custom(BadRequest(format!(
                        "invalid parent 1: expected a hex-string of length {}",
                        MESSAGE_ID_LENGTH * 2
                    ))))?
                    .parse::<MessageId>()
                    .map_err(|_| {
                        reject::custom(BadRequest(format!(
                            "invalid parent 1: expected a hex-string of length {}",
                            MESSAGE_ID_LENGTH * 2
                        )))
                    })?
            } else {
                parent_2_v
                    .as_str()
                    .ok_or(reject::custom(BadRequest(format!(
                        "invalid parent 2: expected a hex-string of length {}",
                        MESSAGE_ID_LENGTH * 2
                    ))))?
                    .parse::<MessageId>()
                    .map_err(|_| {
                        reject::custom(BadRequest(format!(
                            "invalid parent 2: expected a hex-string of length {}",
                            MESSAGE_ID_LENGTH * 2
                        )))
                    })?
            };
            (parent, parent)
        }
    };

    let payload = {
        let payload_v = &value["payload"];
        if payload_v.is_null() {
            None
        } else {
            let payload_dto = serde_json::from_value::<PayloadDto>(payload_v.clone())
                .map_err(|e| reject::custom(BadRequest(e.to_string())))?;
            Some(Payload::try_from(&payload_dto).map_err(|e| reject::custom(BadRequest(e.to_string())))?)
        }
    };

    let nonce = {
        let nonce_v = &value["nonce"];
        if nonce_v.is_null() {
            if !rest_api_config.feature_proof_of_work() {
                return Err(reject::custom(ServiceUnavailable(
                    "can not auto-fill nonce: feature `PoW` not enabled".to_string(),
                )));
            }
            None
        } else {
            Some(
                nonce_v
                    .as_str()
                    .ok_or(reject::custom(BadRequest(
                        "invalid nonce: expected an u64-string".to_string(),
                    )))?
                    .parse::<u64>()
                    .map_err(|_| reject::custom(BadRequest("invalid nonce: expected an u64-string".to_string())))?,
            )
        }
    };

    let message = {
        if let Some(nonce) = nonce {
            let mut builder = MessageBuilder::new()
                .with_network_id(network_id)
                .with_parent1(parent_1_message_id)
                .with_parent2(parent_2_message_id)
                .with_nonce_provider(ConstantBuilder::new().with_value(nonce).finish(), 0f64);
            if let Some(payload) = payload {
                builder = builder.with_payload(payload)
            }
            builder
                .finish()
                .map_err(|e| reject::custom(BadRequest(e.to_string())))?
        } else {
            let mut builder = MessageBuilder::new()
                .with_network_id(network_id)
                .with_parent1(parent_1_message_id)
                .with_parent2(parent_2_message_id)
                .with_nonce_provider(
                    MinerBuilder::new().with_num_workers(num_cpus::get()).finish(),
                    protocol_config.minimum_pow_score(),
                );
            if let Some(payload) = payload {
                builder = builder.with_payload(payload)
            }
            builder
                .finish()
                .map_err(|e| reject::custom(BadRequest(e.to_string())))?
        }
    };

    let message_id = forward_to_message_submitter(message, tangle, message_submitter).await?;

    Ok(warp::reply::with_status(
        warp::reply::json(&SuccessEnvelope::new(PostMessageResponse {
            message_id: message_id.to_string(),
        })),
        StatusCode::CREATED,
    ))
}

pub(crate) async fn forward_to_message_submitter<B: Backend>(
    message: Message,
    tangle: ResHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> Result<MessageId, Rejection> {
    let message_bytes = message.pack_new();
    let message_id = hash_message(&message_bytes);

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

fn hash_message(bytes: &[u8]) -> MessageId {
    use blake2::digest::{Update, VariableOutput};

    let mut blake2b = VarBlake2b::new(MESSAGE_ID_LENGTH).unwrap();
    blake2b.update(bytes);
    let mut bytes = [0u8; 32];
    // TODO Do we have to copy ?
    blake2b.finalize_variable_reset(|digest| bytes.copy_from_slice(&digest));
    MessageId::from(bytes)
}

/// Response of POST /api/v1/messages
#[derive(Clone, Debug, Serialize)]
pub struct PostMessageResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
}

impl EnvelopeContent for PostMessageResponse {}
