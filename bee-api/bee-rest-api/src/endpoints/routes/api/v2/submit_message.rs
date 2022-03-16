// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::{RestApiConfig, ROUTE_SUBMIT_MESSAGE, ROUTE_SUBMIT_MESSAGE_RAW},
        permission::has_permission,
        rejection::CustomRejection,
        storage::StorageBackend,
        NetworkId,
    },
    types::{dtos::PayloadDto, responses::SubmitMessageResponse},
};

use bee_message::{parent::Parents, payload::Payload, Message, MessageBuilder, MessageId};
use bee_pow::providers::{miner::MinerBuilder, NonceProviderBuilder};
use bee_protocol::{
    workers::{config::ProtocolConfig, MessageSubmitterError, MessageSubmitterWorkerEvent},
    PROTOCOL_VERSION,
};
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;

use futures::channel::oneshot;
use log::error;
use packable::PackableExt;
use serde_json::Value as JsonValue;
use tokio::sync::mpsc;
use warp::{ http::StatusCode, reject, Filter, Rejection, Reply};

use std::net::IpAddr;

use axum::extract::{Extension, TypedHeader};
use crate::endpoints::ApiArgsFullNode;
use axum::extract::Json;
use axum::Router;
use axum::routing::post;
use axum::response::IntoResponse;
use crate::endpoints::error::ApiError;
use std::sync::Arc;
use axum::extract::Path;
use axum::http::header::{HeaderMap, HeaderValue};
use axum::response::Response;
use axum::body::Bytes;


pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new()
        .route("/messages", post(submit_message::<B>))
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
            submit_message_json::<B>(serde_json::from_slice(&bytes.to_vec()).map_err(|_| ApiError::BadRequest("invalid JSON".to_string()))?, args.clone()).await
        }
    } else {
        submit_message_json::<B>(serde_json::from_slice(&bytes.to_vec()).map_err(|_| ApiError::BadRequest("invalid JSON".to_string()))?, args.clone()).await
    }
}

pub(crate) async fn submit_message_json<B: StorageBackend>(
    value: JsonValue,
    args: Arc<ApiArgsFullNode<B>>,
) -> Result<Response, ApiError> {
    let network_id_v = &value["networkId"];
    let parents_v = &value["parentMessageIds"];
    let payload_v = &value["payload"];
    let nonce_v = &value["nonce"];

    // parse the values, take care of missing fields, build the message and submit it to the node for further
    // processing

    let network_id = if network_id_v.is_null() {
        args.network_id.1
    } else {
        network_id_v
            .as_str()
            .ok_or_else(|| {
                ApiError::BadRequest(
                    "invalid network id: expected an u64-string".to_string(),
                )
            })?
            .parse::<u64>()
            .map_err(|_| {
                ApiError::BadRequest(
                    "invalid network id: expected an u64-string".to_string(),
                )
            })?
    };

    let parents: Vec<MessageId> = if parents_v.is_null() {
        args.tangle.get_messages_to_approve().await.ok_or_else(|| {
            ApiError::ServiceUnavailable(
                "can not auto-fill parents: no tips available".to_string(),
            )
        })?
    } else {
        let array = parents_v.as_array().ok_or_else(|| {
            ApiError::BadRequest(
                "invalid parents: expected an array of message ids".to_string(),
            )
        })?;
        let mut message_ids = Vec::with_capacity(array.len());
        for s in array {
            let message_id = s
                .as_str()
                .ok_or_else(|| {
                    ApiError::BadRequest(
                        "invalid parents: expected an array of message ids".to_string(),
                    )
                })?
                .parse::<MessageId>()
                .map_err(|_| {
                    ApiError::BadRequest(
                        "invalid network id: expected an u64-string".to_string(),
                    )
                })?;
            message_ids.push(message_id);
        }
        message_ids
    };

    let payload = if payload_v.is_null() {
        None
    } else {
        let payload_dto = serde_json::from_value::<PayloadDto>(payload_v.clone())
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
        Some(Payload::try_from(&payload_dto).map_err(|e| ApiError::BadRequest(e.to_string()))?)
    };

    let nonce = if nonce_v.is_null() {
        None
    } else {
        let parsed = nonce_v
            .as_str()
            .ok_or_else(|| {
                ApiError::BadRequest(
                    "invalid nonce: expected an u64-string".to_string(),
                )
            })?
            .parse::<u64>()
            .map_err(|_| {
                ApiError::BadRequest(
                    "invalid nonce: expected an u64-string".to_string(),
                )
            })?;
        if parsed == 0 { None } else { Some(parsed) }
    };

    let message = build_message(network_id, parents, payload, nonce, args.clone()).await?;
    let message_id = forward_to_message_submitter(message, args).await?;

    Ok((StatusCode::CREATED, Json(SubmitMessageResponse {
            message_id: message_id.to_string(),
        })

    ).into_response())
}

// TODO compare/set network ID and protocol version
pub(crate) async fn build_message<B: StorageBackend>(
    _network_id: u64,
    parents: Vec<MessageId>,
    payload: Option<Payload>,
    nonce: Option<u64>,
    args: Arc<ApiArgsFullNode<B>>,
) -> Result<Message, ApiError> {
    let message = if let Some(nonce) = nonce {
        let mut builder = MessageBuilder::new(
            Parents::new(parents).map_err(|e| ApiError::BadRequest(e.to_string()))?,
        )
        .with_protocol_version(PROTOCOL_VERSION)
        .with_nonce_provider(nonce, 0f64);
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder
            .finish()
            .map_err(|e| ApiError::BadRequest(e.to_string()))?
    } else {
        if !args.rest_api_config.feature_proof_of_work() {
            return Err(ApiError::ServiceUnavailable(
                "can not auto-fill nonce: feature `PoW` not enabled".to_string(),
            ));
        }
        let mut builder = MessageBuilder::new(
            Parents::new(parents).map_err(|e| ApiError::BadRequest(e.to_string()))?,
        )
        .with_protocol_version(PROTOCOL_VERSION)
        .with_nonce_provider(
            MinerBuilder::new().with_num_workers(num_cpus::get()).finish(),
            args.protocol_config.minimum_pow_score(),
        );
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder
            .finish()
            .map_err(|e| ApiError::BadRequest(e.to_string()))?
    };
    Ok(message)
}

pub(crate) async fn submit_message_raw<B: StorageBackend>(
    bytes: Vec<u8>,
    args: Arc<ApiArgsFullNode<B>>,
) -> Result<Response, ApiError> {
    let message = Message::unpack_verified(&bytes).map_err(|_| {
        ApiError::BadRequest(
            "can not submit message: invalid bytes provided: the message format is not respected".to_string(),
        )
    })?;
    let message_id = forward_to_message_submitter(message, args).await?;
    Ok((StatusCode::CREATED, Json(SubmitMessageResponse {
            message_id: message_id.to_string(),
        }),
    ).into_response())
}

pub(crate) async fn forward_to_message_submitter<B: StorageBackend>(
    message: Message,
    args: Arc<ApiArgsFullNode<B>>,
) -> Result<MessageId, ApiError> {
    let message_id = message.id();
    let message_bytes = message.pack_to_vec();

    if args.tangle.contains(&message_id).await {
        return Ok(message_id);
    }

    let (notifier, waiter) = oneshot::channel::<Result<MessageId, MessageSubmitterError>>();

    args.message_submitter
        .send(MessageSubmitterWorkerEvent {
            message: message_bytes,
            notifier,
        })
        .map_err(|e| {
            error!("can not submit message: {}", e);
            ApiError::ServiceUnavailable(
                "can not submit message".to_string(),
            )
        })?;

    match waiter.await.map_err(|e| {
        error!("can not submit message: {}", e);
        ApiError::ServiceUnavailable(
            "can not submit message".to_string(),
        )
    })? {
        Ok(message_id) => Ok(message_id),
        Err(e) => Err(ApiError::BadRequest(format!(
            "can not submit message: message is invalid: {}",
            e
        ))),
    }
}
