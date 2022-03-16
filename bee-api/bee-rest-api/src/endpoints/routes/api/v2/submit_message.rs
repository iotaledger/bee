// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::storage::StorageBackend,
    types::{dtos::PayloadDto, responses::SubmitMessageResponse},
};

use bee_message::{parent::Parents, payload::Payload, Message, MessageBuilder, MessageId};
use bee_pow::providers::{miner::MinerBuilder, NonceProviderBuilder};
use bee_protocol::{
    workers::{MessageSubmitterError, MessageSubmitterWorkerEvent},
    PROTOCOL_VERSION,
};

use futures::channel::oneshot;
use log::error;
use packable::PackableExt;
use serde_json::Value as JsonValue;

use crate::endpoints::{error::ApiError, ApiArgsFullNode};
use axum::{
    body::Bytes,
    extract::{Extension, Json},
    http::{
        header::{HeaderMap, HeaderValue},
        StatusCode,
    },
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use std::sync::Arc;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/messages", post(submit_message::<B>))
}

pub(crate) async fn submit_message<B: StorageBackend>(
    bytes: Bytes,
    headers: HeaderMap,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<Response, ApiError> {
    if let Some(value) = headers.get(axum::http::header::CONTENT_TYPE) {
        if value.eq(&"application/octet-stream".parse::<HeaderValue>().unwrap()) {
            submit_message_raw::<B>(bytes.to_vec(), args.clone()).await
        } else {
            submit_message_json::<B>(
                serde_json::from_slice(&bytes.to_vec())
                    .map_err(|_| ApiError::BadRequest("invalid JSON".to_string()))?,
                args.clone(),
            )
            .await
        }
    } else {
        submit_message_json::<B>(
            serde_json::from_slice(&bytes.to_vec()).map_err(|_| ApiError::BadRequest("invalid JSON".to_string()))?,
            args.clone(),
        )
        .await
    }
}

pub(crate) async fn submit_message_json<B: StorageBackend>(
    value: JsonValue,
    args: Arc<ApiArgsFullNode<B>>,
) -> Result<Response, ApiError> {
    let protocol_version_json = &value["protocolVersion"];
    let parents_json = &value["parentMessageIds"];
    let payload_json = &value["payload"];
    let nonce_json = &value["nonce"];

    // Tries to build a `Message` from the given JSON.
    // If some fields are missing, it tries to auto-complete them.

    if !protocol_version_json.is_null() {
        let parsed_protocol_version = protocol_version_json.as_u64().ok_or_else(|| {
            reject::custom(CustomRejection::BadRequest(
                "invalid protocol version: expected an unsigned integer < 256".to_string(),
            ))
        })? as u8;

        if parsed_protocol_version != PROTOCOL_VERSION {
            return Err(reject::custom(CustomRejection::BadRequest(format!(
                "invalid protocol version: expected `{}` but received `{}`",
                PROTOCOL_VERSION, parsed_protocol_version
            ))));
        }
    }

    let parents: Vec<MessageId> = if parents_json.is_null() {
        tangle.get_messages_to_approve().await.ok_or_else(|| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not auto-fill parents: no tips available".to_string(),
            ))
        })?
    } else {
        let parents = parents_json.as_array().ok_or_else(|| {
            reject::custom(CustomRejection::BadRequest(
                "invalid parents: expected an array of message ids".to_string(),
            ))
        })?;
        let mut message_ids = Vec::with_capacity(parents.len());
        for message_id in parents {
            let message_id = message_id
                .as_str()
                .ok_or_else(|| {
                    reject::custom(CustomRejection::BadRequest(
                        "invalid parent: expected a message id".to_string(),
                    ))
                })?
                .parse::<MessageId>()
                .map_err(|_| {
                    reject::custom(CustomRejection::BadRequest(
                        "invalid parent: expected a message id".to_string(),
                    ))
                })?;
            message_ids.push(message_id);
        }
        message_ids
    };

    let payload = if payload_json.is_null() {
        None
    } else {
        let payload_dto = serde_json::from_value::<PayloadDto>(payload_json.clone())
            .map_err(|e| reject::custom(CustomRejection::BadRequest(e.to_string())))?;
        Some(Payload::try_from(&payload_dto).map_err(|e| reject::custom(CustomRejection::BadRequest(e.to_string())))?)
    };

    let nonce = if nonce_json.is_null() {
        None
    } else {
        let parsed_nonce = nonce_json
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
        if parsed_nonce == 0 { None } else { Some(parsed_nonce) }
    };

    let message = build_message(parents, payload, nonce, args.clone()).await?;
    let message_id = forward_to_message_submitter(message.pack_to_vec(), args).await?;

    Ok((
        StatusCode::CREATED,
        Json(SubmitMessageResponse {
            message_id: message_id.to_string(),
        }),
    )
        .into_response())
}

pub(crate) async fn build_message(
    parents: Vec<MessageId>,
    payload: Option<Payload>,
    nonce: Option<u64>,
    args: Arc<ApiArgsFullNode<B>>,
) -> Result<Message, ApiError> {
    let message = if let Some(nonce) = nonce {
        let mut builder = MessageBuilder::new(Parents::new(parents).map_err(|e| ApiError::BadRequest(e.to_string()))?)
            .with_protocol_version(PROTOCOL_VERSION)
            .with_nonce_provider(nonce, 0f64);
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder.finish().map_err(|e| ApiError::BadRequest(e.to_string()))?
    } else {
        if !args.rest_api_config.feature_proof_of_work() {
            return Err(ApiError::ServiceUnavailable(
                "can not auto-fill nonce: feature `PoW` not enabled".to_string(),
            ));
        }
        let mut builder = MessageBuilder::new(Parents::new(parents).map_err(|e| ApiError::BadRequest(e.to_string()))?)
            .with_protocol_version(PROTOCOL_VERSION)
            .with_nonce_provider(
                MinerBuilder::new().with_num_workers(num_cpus::get()).finish(),
                args.protocol_config.minimum_pow_score(),
            );
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder.finish().map_err(|e| ApiError::BadRequest(e.to_string()))?
    };
    Ok(message)
}

pub(crate) async fn submit_message_raw<B: StorageBackend>(
    message_bytes: Vec<u8>,
    args: Arc<ApiArgsFullNode<B>>,
) -> Result<Response, ApiError> {
    let message_id = forward_to_message_submitter(message_bytes, args).await?;
    Ok((
        StatusCode::CREATED,
        Json(SubmitMessageResponse {
            message_id: message_id.to_string(),
        }),
    )
        .into_response())
}

pub(crate) async fn forward_to_message_submitter<B: StorageBackend>(
    message_bytes: Vec<u8>,
    args: Arc<ApiArgsFullNode<B>>,
) -> Result<MessageId, ApiError> {
    let (notifier, waiter) = oneshot::channel::<Result<MessageId, MessageSubmitterError>>();

    args.message_submitter
        .send(MessageSubmitterWorkerEvent {
            message: message_bytes,
            notifier,
        })
        .map_err(|e| {
            error!("can not submit message: {}", e);
            ApiError::ServiceUnavailable("can not submit message".to_string())
        })?;

    match waiter.await.map_err(|e| {
        error!("can not submit message: {}", e);
        ApiError::ServiceUnavailable("can not submit message".to_string())
    })? {
        Ok(message_id) => Ok(message_id),
        Err(e) => Err(ApiError::BadRequest(format!(
            "can not submit message: message is invalid: {}",
            e
        ))),
    }
}
