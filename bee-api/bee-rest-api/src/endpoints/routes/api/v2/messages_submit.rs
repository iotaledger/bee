// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    body::Bytes,
    extract::{Extension, Json},
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use bee_message::{
    constant::PROTOCOL_VERSION,
    parent::Parents,
    payload::{dto::PayloadDto, Payload},
    Message, MessageBuilder, MessageId,
};
use bee_pow::providers::{miner::MinerBuilder, NonceProviderBuilder};
use bee_protocol::workers::{MessageSubmitterError, MessageSubmitterWorkerEvent};
use futures::channel::oneshot;
use log::error;
use packable::PackableExt;
use serde_json::Value;

use crate::{
    endpoints::{
        error::ApiError, routes::api::v2::messages::BYTE_CONTENT_TYPE, storage::StorageBackend, ApiArgsFullNode,
    },
    types::responses::SubmitMessageResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/messages", post(messages_submit::<B>))
}

async fn messages_submit<B: StorageBackend>(
    bytes: Bytes,
    headers: HeaderMap,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<Response, ApiError> {
    if let Some(value) = headers.get(axum::http::header::CONTENT_TYPE) {
        if value.eq(&*BYTE_CONTENT_HEADER) {
            submit_message_raw::<B>(bytes.to_vec(), args.clone()).await
        } else {
            submit_message_json::<B>(
                serde_json::from_slice(&bytes.to_vec()).map_err(ApiError::InvalidJsonProvided)?,
                args.clone(),
            )
            .await
        }
    } else {
        submit_message_json::<B>(
            serde_json::from_slice(&bytes.to_vec()).map_err(ApiError::InvalidJsonProvided)?,
            args.clone(),
        )
        .await
    }
}

pub(crate) async fn submit_message_json<B: StorageBackend>(
    value: Value,
    args: ApiArgsFullNode<B>,
) -> Result<Response, ApiError> {
    let protocol_version_json = &value["protocolVersion"];
    let parents_json = &value["parentMessageIds"];
    let payload_json = &value["payload"];
    let nonce_json = &value["nonce"];

    // Tries to build a `Message` from the given JSON.
    // If some fields are missing, it tries to auto-complete them.

    if !protocol_version_json.is_null() {
        let parsed_protocol_version = u8::try_from(protocol_version_json.as_u64().ok_or(ApiError::BadRequest(
            "invalid protocol version: expected an unsigned integer < 256",
        ))?)
        .map_err(|_| ApiError::BadRequest("invalid protocol version: expected an unsigned integer < 256"))?;

        if parsed_protocol_version != PROTOCOL_VERSION {
            return Err(ApiError::BadRequest("invalid protocol version"));
        }
    }

    let parents: Vec<MessageId> = if parents_json.is_null() {
        let mut parents = args
            .tangle
            .get_messages_to_approve()
            .await
            .ok_or(ApiError::ServiceUnavailable(
                "can not auto-fill parents: no tips available",
            ))?;
        parents.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
        parents
    } else {
        let parents = parents_json.as_array().ok_or(ApiError::BadRequest(
            "invalid parents: expected an array of message ids",
        ))?;
        let mut message_ids = Vec::with_capacity(parents.len());
        for message_id in parents {
            let message_id = message_id
                .as_str()
                .ok_or(ApiError::BadRequest("invalid parent: expected a message id"))?
                .parse::<MessageId>()
                .map_err(|_| ApiError::BadRequest("invalid parent: expected a message id"))?;
            message_ids.push(message_id);
        }
        message_ids
    };

    let payload = if payload_json.is_null() {
        None
    } else {
        let payload_dto =
            serde_json::from_value::<PayloadDto>(payload_json.clone()).map_err(ApiError::InvalidJsonProvided)?;
        Some(Payload::try_from(&payload_dto).map_err(ApiError::InvalidDto)?)
    };

    let nonce = if nonce_json.is_null() {
        None
    } else {
        let parsed_nonce = nonce_json
            .as_str()
            .ok_or(ApiError::BadRequest("invalid nonce: expected an u64-string"))?
            .parse::<u64>()
            .map_err(|_| ApiError::BadRequest("invalid nonce: expected an u64-string"))?;
        if parsed_nonce == 0 { None } else { Some(parsed_nonce) }
    };

    let message = build_message(parents, payload, nonce, args.clone())?;
    let message_id = forward_to_message_submitter(message.pack_to_vec(), args).await?;

    Ok((
        StatusCode::CREATED,
        Json(SubmitMessageResponse {
            message_id: message_id.to_string(),
        }),
    )
        .into_response())
}

pub(crate) fn build_message<B: StorageBackend>(
    parents: Vec<MessageId>,
    payload: Option<Payload>,
    nonce: Option<u64>,
    args: ApiArgsFullNode<B>,
) -> Result<Message, ApiError> {
    let message = if let Some(nonce) = nonce {
        let mut builder = MessageBuilder::new(Parents::new(parents).map_err(ApiError::InvalidMessage)?)
            .with_nonce_provider(nonce, 0f64);
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder.finish().map_err(ApiError::InvalidMessage)?
    } else {
        if !args.rest_api_config.feature_proof_of_work() {
            return Err(ApiError::BadRequest(
                "can not auto-fill nonce: feature `PoW` not enabled",
            ));
        }
        let mut builder = MessageBuilder::new(Parents::new(parents).map_err(ApiError::InvalidMessage)?)
            .with_nonce_provider(
                MinerBuilder::new().with_num_workers(num_cpus::get()).finish(),
                args.protocol_config.minimum_pow_score(),
            );
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder.finish().map_err(ApiError::InvalidMessage)?
    };
    Ok(message)
}

pub(crate) async fn submit_message_raw<B: StorageBackend>(
    message_bytes: Vec<u8>,
    args: ApiArgsFullNode<B>,
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
    args: ApiArgsFullNode<B>,
) -> Result<MessageId, ApiError> {
    let (notifier, waiter) = oneshot::channel::<Result<MessageId, MessageSubmitterError>>();

    args.message_submitter
        .send(MessageSubmitterWorkerEvent {
            message: message_bytes,
            notifier,
        })
        .map_err(|e| {
            error!("cannot submit message: {}", e);
            ApiError::InternalError
        })?;

    let result = waiter.await.map_err(|e| {
        error!("cannot submit message: {}", e);
        ApiError::InternalError
    })?;

    result.map_err(ApiError::InvalidMessageSubmitted)
}
