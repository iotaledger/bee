// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    any::TypeId,
    collections::HashSet,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::post,
    Router,
};
use bee_block::{payload::milestone::MilestoneIndex, BlockId};
use bee_ledger::workers::consensus::{self, WhiteFlagMetadata};
use bee_protocol::workers::{event::BlockSolidified, request_block};
use futures::channel::oneshot;
use serde_json::Value;
use tokio::time::timeout;

use crate::{
    endpoints::{error::ApiError, extractors::json::CustomJson, storage::StorageBackend, ApiArgsFullNode},
    types::responses::WhiteFlagResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/whiteflag", post(white_flag::<B>))
}

pub(crate) async fn white_flag<B: StorageBackend>(
    CustomJson(body): CustomJson<Value>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let index_json = &body["index"];
    let parents_json = &body["parents"];

    let index = if index_json.is_null() {
        return Err(ApiError::BadRequest("invalid index: expected a `MilestoneIndex`"));
    } else {
        MilestoneIndex(
            index_json
                .as_u64()
                .ok_or(ApiError::BadRequest("invalid index: expected a `MilestoneIndex`"))? as u32,
        )
    };

    let parents = if parents_json.is_null() {
        return Err(ApiError::BadRequest("invalid parents: expected an array of `BlockId`"));
    } else {
        let array = parents_json
            .as_array()
            .ok_or(ApiError::BadRequest("invalid parents: expected an array of `BlockId`"))?;
        let mut block_ids = Vec::new();
        for s in array {
            let block_id = s
                .as_str()
                .ok_or(ApiError::BadRequest("invalid parents: expected an array of `BlockId`"))?
                .parse::<BlockId>()
                .map_err(|_| ApiError::BadRequest("invalid parents: expected an array of `BlockId`"))?;
            block_ids.push(block_id);
        }
        block_ids
    };

    // TODO check parents

    // White flag is usually called on solid milestones; however, this endpoint's purpose is to provide the coordinator
    // with the node's perception of a milestone candidate before issuing it. Within this endpoint, there is then no
    // guarantee that the provided parents are solid so the node needs to solidify them before calling white flag or
    // aborting if it took too long. This is done by requesting all missing parents then listening for their
    // solidification event or aborting if the allowed time passed.

    let to_solidify = Arc::new(Mutex::new(parents.iter().copied().collect::<HashSet<BlockId>>()));
    let (sender, receiver) = oneshot::channel::<()>();
    let sender = Arc::new(Mutex::new(Some(sender)));

    // Start listening to solidification events to check if the parents are getting solid.
    let task_to_solidify = to_solidify.clone();
    let task_sender = sender.clone();
    struct Static;
    args.bus.add_listener::<Static, _, _>(move |event: &BlockSolidified| {
        if let Ok(mut to_solidify) = task_to_solidify.lock() {
            if to_solidify.remove(&event.block_id) && to_solidify.is_empty() {
                if let Ok(mut sender) = task_sender.lock() {
                    sender.take().map(|s| s.send(()));
                }
            }
        }
    });

    for parent in parents.iter() {
        if args.tangle.is_solid_block(parent).await {
            if let Ok(mut to_solidify) = to_solidify.lock() {
                to_solidify.remove(parent);
            }
        } else {
            request_block(
                &*args.tangle,
                &args.block_requester,
                &*args.requested_blocks,
                *parent,
                index,
            )
            .await;
        }
    }

    if let Ok(to_solidify) = to_solidify.lock() {
        if to_solidify.is_empty() {
            if let Ok(mut sender) = sender.lock() {
                sender.take().map(|s| s.send(()));
            }
        }
    }

    // TODO Actually pass the previous milestone id ?
    let mut metadata = WhiteFlagMetadata::new(index, 0, None);

    // Wait for all parents to get solid or the timeout to expire.
    let response = match timeout(args.rest_api_config.white_flag_solidification_timeout(), receiver).await {
        Ok(_) => {
            // Did not timeout, parents are solid and white flag can happen.
            consensus::white_flag::<B>(&args.tangle, &args.storage, &parents, &mut metadata)
                .await
                .map_err(ApiError::InvalidWhiteflag)?;

            Ok(Json(WhiteFlagResponse {
                merkle_tree_hash: metadata.applied_merkle_root().to_string(),
            }))
        }
        Err(_) => {
            // Did timeout, parents are not solid and white flag can not happen.
            Err(ApiError::ServiceUnavailable("parents not solid"))
        }
    };

    // Stop listening to the solidification event.
    args.bus.remove_listeners_by_id(TypeId::of::<Static>());

    response
}
