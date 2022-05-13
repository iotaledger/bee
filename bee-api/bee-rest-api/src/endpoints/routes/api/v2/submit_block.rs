// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_block::{
    constant::PROTOCOL_VERSION,
    parent::Parents,
    payload::{dto::PayloadDto, Payload},
    Block, BlockBuilder, BlockId,
};
use bee_pow::providers::{miner::MinerBuilder, NonceProviderBuilder};
use bee_protocol::workers::{config::ProtocolConfig, BlockSubmitterError, BlockSubmitterWorkerEvent};
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use futures::channel::oneshot;
use log::error;
use packable::PackableExt;
use serde_json::Value as JsonValue;
use tokio::sync::mpsc;
use warp::{filters::BoxedFilter, http::StatusCode, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::{RestApiConfig, ROUTE_SUBMIT_MESSAGE, ROUTE_SUBMIT_MESSAGE_RAW},
        filters::{with_block_submitter, with_protocol_config, with_rest_api_config, with_tangle},
        permission::has_permission,
        rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::responses::SubmitBlockResponse,
};

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("blocks")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
    block_submitter: mpsc::UnboundedSender<BlockSubmitterWorkerEvent>,
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
                .and(with_tangle(tangle))
                .and(with_block_submitter(block_submitter.clone()))
                .and(with_rest_api_config(rest_api_config))
                .and(with_protocol_config(protocol_config))
                .and_then(submit_block))
            .or(warp::header::exact("content-type", "application/octet-stream")
                .and(has_permission(ROUTE_SUBMIT_MESSAGE_RAW, public_routes, allowed_ips))
                .and(warp::body::bytes())
                .and(with_block_submitter(block_submitter))
                .and_then(submit_block_raw)),
        )
        .boxed()
}

pub(crate) async fn submit_block<B: StorageBackend>(
    value: JsonValue,
    tangle: ResourceHandle<Tangle<B>>,
    block_submitter: mpsc::UnboundedSender<BlockSubmitterWorkerEvent>,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> Result<impl Reply, Rejection> {
    let protocol_version_json = &value["protocolVersion"];
    let parents_json = &value["parentBlockIds"];
    let payload_json = &value["payload"];
    let nonce_json = &value["nonce"];

    // Tries to build a `Block` from the given JSON.
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

    let parents: Vec<BlockId> = if parents_json.is_null() {
        tangle.get_blocks_to_approve().await.ok_or_else(|| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not auto-fill parents: no tips available".to_string(),
            ))
        })?
    } else {
        let parents = parents_json.as_array().ok_or_else(|| {
            reject::custom(CustomRejection::BadRequest(
                "invalid parents: expected an array of block ids".to_string(),
            ))
        })?;
        let mut block_ids = Vec::with_capacity(parents.len());
        for block_id in parents {
            let block_id = block_id
                .as_str()
                .ok_or_else(|| {
                    reject::custom(CustomRejection::BadRequest(
                        "invalid parent: expected a block id".to_string(),
                    ))
                })?
                .parse::<BlockId>()
                .map_err(|_| {
                    reject::custom(CustomRejection::BadRequest(
                        "invalid parent: expected a block id".to_string(),
                    ))
                })?;
            block_ids.push(block_id);
        }
        block_ids
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

    let block = build_block(parents, payload, nonce, rest_api_config, protocol_config)?;
    let block_id = forward_to_block_submitter(block.pack_to_vec(), block_submitter).await?;

    Ok(warp::reply::with_status(
        warp::reply::json(&SubmitBlockResponse {
            block_id: block_id.to_string(),
        }),
        StatusCode::CREATED,
    ))
}

pub(crate) fn build_block(
    parents: Vec<BlockId>,
    payload: Option<Payload>,
    nonce: Option<u64>,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> Result<Block, Rejection> {
    let block = if let Some(nonce) = nonce {
        let mut builder = BlockBuilder::new(
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
        let mut builder = BlockBuilder::new(
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
    Ok(block)
}

pub(crate) async fn submit_block_raw(
    buf: warp::hyper::body::Bytes,
    block_submitter: mpsc::UnboundedSender<BlockSubmitterWorkerEvent>,
) -> Result<impl Reply, Rejection> {
    let block_id = forward_to_block_submitter((*buf).to_vec(), block_submitter).await?;
    Ok(warp::reply::with_status(
        warp::reply::json(&SubmitBlockResponse {
            block_id: block_id.to_string(),
        }),
        StatusCode::CREATED,
    ))
}

pub(crate) async fn forward_to_block_submitter(
    block_bytes: Vec<u8>,
    block_submitter: mpsc::UnboundedSender<BlockSubmitterWorkerEvent>,
) -> Result<BlockId, Rejection> {
    let (notifier, waiter) = oneshot::channel::<Result<BlockId, BlockSubmitterError>>();

    block_submitter
        .send(BlockSubmitterWorkerEvent {
            block: block_bytes,
            notifier,
        })
        .map_err(|e| {
            error!("can not submit block: {}", e);
            reject::custom(CustomRejection::ServiceUnavailable("can not submit block".to_string()))
        })?;

    match waiter.await.map_err(|e| {
        error!("can not submit block: {}", e);
        reject::custom(CustomRejection::ServiceUnavailable("can not submit block".to_string()))
    })? {
        Ok(block_id) => Ok(block_id),
        Err(e) => Err(reject::custom(CustomRejection::BadRequest(format!(
            "can not submit block: block is invalid: {}",
            e
        )))),
    }
}
