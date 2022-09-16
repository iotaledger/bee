// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{body::Bytes, extract::Extension, http::header::HeaderMap, routing::post, Router};
use bee_block::{
    parent::Parents,
    payload::{
        dto::{try_from_payload_dto_payload, PayloadDto},
        Payload,
    },
    protocol::ProtocolParameters,
    Block, BlockBuilder, BlockId,
};
use bee_pow::providers::{miner::MinerBuilder, NonceProviderBuilder};
use bee_protocol::{BlockSubmitterError, BlockSubmitterWorkerEvent};
use futures::channel::oneshot;
use log::error;
use packable::PackableExt;
use serde_json::Value;

use crate::{
    error::{ApiError, DependencyError},
    routes::api::core::v2::blocks::BYTE_CONTENT_HEADER,
    storage::StorageBackend,
    types::responses::SubmitBlockResponse,
    ApiArgsFullNode,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/blocks", post(blocks_submit::<B>))
}

async fn blocks_submit<B: StorageBackend>(
    bytes: Bytes,
    headers: HeaderMap,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<SubmitBlockResponse, ApiError> {
    if let Some(value) = headers.get(axum::http::header::CONTENT_TYPE) {
        if value.eq(&*BYTE_CONTENT_HEADER) {
            return submit_block_raw::<B>(bytes.to_vec(), args.clone()).await;
        }
    }
    submit_block_json::<B>(
        serde_json::from_slice(&bytes).map_err(|e| ApiError::DependencyError(DependencyError::SerdeJsonError(e)))?,
        args.clone(),
    )
    .await
}

pub(crate) async fn submit_block_json<B: StorageBackend>(
    value: Value,
    args: ApiArgsFullNode<B>,
) -> Result<SubmitBlockResponse, ApiError> {
    // TODO: this is obviously wrong but can't be done properly until the snapshot PR is merged.
    // The node can't work properly with this.
    // @thibault-martinez.
    let protocol_parameters = ProtocolParameters::default();

    let protocol_version_json = &value["protocolVersion"];
    let parents_json = &value["parents"];
    let payload_json = &value["payload"];
    let nonce_json = &value["nonce"];

    // Tries to build a `Block` from the given JSON.
    // If some fields are missing, it tries to auto-complete them.

    if !protocol_version_json.is_null() {
        let parsed_protocol_version = u8::try_from(protocol_version_json.as_u64().ok_or(ApiError::BadRequest(
            "invalid protocol version: expected an unsigned integer < 256",
        ))?)
        .map_err(|_| ApiError::BadRequest("invalid protocol version: expected an unsigned integer < 256"))?;

        if parsed_protocol_version != protocol_parameters.protocol_version() {
            return Err(ApiError::BadRequest("invalid protocol version"));
        }
    }

    let parents: Vec<BlockId> = if parents_json.is_null() {
        let mut parents = args
            .tangle
            .get_blocks_to_approve()
            .await
            .ok_or(ApiError::ServiceUnavailable(
                "can not auto-fill parents: no tips available",
            ))?;
        parents.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
        parents
    } else {
        let parents = parents_json
            .as_array()
            .ok_or(ApiError::BadRequest("invalid parents: expected an array of block ids"))?;
        let mut block_ids = Vec::with_capacity(parents.len());
        for block_id in parents {
            let block_id = block_id
                .as_str()
                .ok_or(ApiError::BadRequest("invalid parent: expected a block id"))?
                .parse::<BlockId>()
                .map_err(|_| ApiError::BadRequest("invalid parent: expected a block id"))?;
            block_ids.push(block_id);
        }
        block_ids
    };

    let payload = if payload_json.is_null() {
        None
    } else {
        let payload_dto = serde_json::from_value::<PayloadDto>(payload_json.clone())
            .map_err(|e| ApiError::DependencyError(DependencyError::SerdeJsonError(e)))?;
        Some(
            try_from_payload_dto_payload(&payload_dto, &protocol_parameters)
                .map_err(|e| ApiError::DependencyError(DependencyError::InvalidDto(e)))?,
        )
    };

    let nonce = if nonce_json.is_null() {
        None
    } else {
        let parsed_nonce = nonce_json
            .as_str()
            .ok_or(ApiError::BadRequest("invalid nonce: expected an u64-string"))?
            .parse::<u64>()
            .map_err(|_| ApiError::BadRequest("invalid nonce: expected an u64-string"))?;

        Some(parsed_nonce)
    };

    let block = build_block(parents, payload, nonce, args.clone())?;
    let block_id = forward_to_block_submitter(block.pack_to_vec(), args).await?;

    Ok(SubmitBlockResponse {
        block_id: block_id.to_string(),
    })
}

pub(crate) fn build_block<B: StorageBackend>(
    parents: Vec<BlockId>,
    payload: Option<Payload>,
    nonce: Option<u64>,
    args: ApiArgsFullNode<B>,
) -> Result<Block, ApiError> {
    // TODO: this is obviously wrong but can't be done properly until the snapshot PR is merged.
    // The node can't work properly with this.
    // @thibault-martinez.
    let protocol_parameters = ProtocolParameters::default();

    let block = if let Some(nonce) = nonce {
        let mut builder = BlockBuilder::new(
            Parents::new(parents).map_err(|e| ApiError::DependencyError(DependencyError::InvalidBlock(e)))?,
        )
        .with_nonce_provider(nonce);
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder
            .finish(protocol_parameters.min_pow_score())
            .map_err(|e| ApiError::DependencyError(DependencyError::InvalidBlock(e)))?
    } else {
        if !args.rest_api_config.feature_proof_of_work() {
            return Err(ApiError::BadRequest(
                "can not auto-fill nonce: feature `PoW` not enabled",
            ));
        }
        let mut builder = BlockBuilder::new(
            Parents::new(parents).map_err(|e| ApiError::DependencyError(DependencyError::InvalidBlock(e)))?,
        )
        .with_nonce_provider(MinerBuilder::new().with_num_workers(num_cpus::get()).finish());
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder
            .finish(protocol_parameters.min_pow_score())
            .map_err(|e| ApiError::DependencyError(DependencyError::InvalidBlock(e)))?
    };
    Ok(block)
}

pub(crate) async fn submit_block_raw<B: StorageBackend>(
    block_bytes: Vec<u8>,
    args: ApiArgsFullNode<B>,
) -> Result<SubmitBlockResponse, ApiError> {
    let block_id = forward_to_block_submitter(block_bytes, args).await?;
    Ok(SubmitBlockResponse {
        block_id: block_id.to_string(),
    })
}

pub(crate) async fn forward_to_block_submitter<B: StorageBackend>(
    block_bytes: Vec<u8>,
    args: ApiArgsFullNode<B>,
) -> Result<BlockId, ApiError> {
    let (notifier, waiter) = oneshot::channel::<Result<BlockId, BlockSubmitterError>>();

    args.block_submitter
        .send(BlockSubmitterWorkerEvent {
            block: block_bytes,
            notifier,
        })
        .map_err(|e| {
            error!("cannot submit block: {}", e);
            ApiError::InternalServerError
        })?;

    let result = waiter.await.map_err(|e| {
        error!("cannot submit block: {}", e);
        ApiError::InternalServerError
    })?;

    result.map_err(|e| ApiError::DependencyError(DependencyError::InvalidBlockSubmitted(e)))
}
