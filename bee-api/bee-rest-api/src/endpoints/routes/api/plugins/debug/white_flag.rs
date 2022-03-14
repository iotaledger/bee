// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::{RestApiConfig, ROUTE_WHITE_FLAG},
        permission::has_permission,
        rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::responses::WhiteFlagResponse,
};

use bee_ledger::workers::consensus::{self, WhiteFlagMetadata};
use bee_message::{milestone::MilestoneIndex, MessageId};
use bee_protocol::workers::{event::MessageSolidified, request_message, MessageRequesterWorker, RequestedMessages};
use bee_runtime::{event::Bus, resource::ResourceHandle};
use bee_tangle::Tangle;

use futures::channel::oneshot;
use serde_json::{Value as JsonValue, Value};
use tokio::time::timeout;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::{
    any::TypeId,
    collections::HashSet,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use axum::extract::Extension;
use crate::endpoints::ApiArgsFullNode;
use axum::extract::Json;
use axum::Router;
use axum::routing::post;
use axum::response::IntoResponse;
use crate::endpoints::error::ApiError;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new()
        .route("/whiteflag", post(white_flag::<B>))
}

pub(crate) async fn white_flag<B: StorageBackend>(
    Json(body): Json<Value>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> impl IntoResponse {
    let index_json = &body["index"];
    let parents_json = &body["parentMessageIds"];

    let index = if index_json.is_null() {
        return ApiError::BadRequest(
            "Invalid index: expected a MilestoneIndex".to_string(),
        );
    } else {
        MilestoneIndex(index_json.as_u64().ok_or_else(|| {
           ApiError::BadRequest(
                "Invalid index: expected a MilestoneIndex".to_string(),
            )
        })? as u32)
    };

    let parents = if parents_json.is_null() {
        return ApiError::BadRequest(
            "Invalid parents: expected an array of MessageId".to_string(),
        );
    } else {
        let array = parents_json.as_array().ok_or_else(|| {
            ApiError::BadRequest(
                "Invalid parents: expected an array of MessageId".to_string(),
            )
        })?;
        let mut message_ids = Vec::new();
        for s in array {
            let message_id = s
                .as_str()
                .ok_or_else(|| {
                    ApiError::BadRequest(
                        "Invalid parents: expected an array of MessageId".to_string(),
                    )
                })?
                .parse::<MessageId>()
                .map_err(|_| {
                   ApiError::BadRequest(
                        "Invalid parents: expected an array of MessageId".to_string(),
                    )
                })?;
            message_ids.push(message_id);
        }
        message_ids
    };

    // TODO check parents

    // White flag is usually called on solid milestones; however, this endpoint's purpose is to provide the coordinator
    // with the node's perception of a milestone candidate before issuing it. Within this endpoint, there is then no
    // guarantee that the provided parents are solid so the node needs to solidify them before calling white flag or
    // aborting if it took too long. This is done by requesting all missing parents then listening for their
    // solidification event or aborting if the allowed time passed.

    let to_solidify = Arc::new(Mutex::new(parents.iter().copied().collect::<HashSet<MessageId>>()));
    let (sender, receiver) = oneshot::channel::<()>();
    let sender = Arc::new(Mutex::new(Some(sender)));

    // Start listening to solidification events to check if the parents are getting solid.
    let task_to_solidify = to_solidify.clone();
    let task_sender = sender.clone();
    struct Static;
    args.bus.add_listener::<Static, _, _>(move |event: &MessageSolidified| {
        if let Ok(mut to_solidify) = task_to_solidify.lock() {
            if to_solidify.remove(&event.message_id) && to_solidify.is_empty() {
                if let Ok(mut sender) = task_sender.lock() {
                    sender.take().map(|s| s.send(()));
                }
            }
        }
    });

    for parent in parents.iter() {
        if args.tangle.is_solid_message(parent).await {
            if let Ok(mut to_solidify) = to_solidify.lock() {
                to_solidify.remove(parent);
            }
        } else {
            request_message(&*args.tangle, &args.message_requester, &*args.requested_messages, *parent, index).await;
        }
    }

    if let Ok(to_solidify) = to_solidify.lock() {
        if to_solidify.is_empty() {
            if let Ok(mut sender) = sender.lock() {
                sender.take().map(|s| s.send(()));
            }
        }
    }

    // TODO
    let mut metadata = WhiteFlagMetadata::new(index, 0);

    // Wait for either all parents to get solid or the timeout to expire.
    let response = match timeout(
        Duration::from_secs(args.rest_api_config.white_flag_solidification_timeout()),
        receiver,
    )
    .await
    {
        Ok(_) => {
            // Did not timeout, parents are solid and white flag can happen.
            consensus::white_flag::<B>(&args.tangle, &args.storage, &parents, &mut metadata)
                .await
                .map_err(|e| ApiError::BadRequest(e.to_string()))?;

            Ok(warp::reply::json(&WhiteFlagResponse {
                merkle_tree_hash: hex::encode(metadata.merkle_proof()),
            }))
        }
        Err(_) => {
            // Did timeout, parents are not solid and white flag can not happen.
            ApiError::ServiceUnavailable(
                "parents not solid".to_string(),
            )
        }
    };

    // Stop listening to the solidification event.
    args.bus.remove_listeners_by_id(TypeId::of::<Static>());

    response
}
