// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::{RestApiConfig, ROUTE_WHITE_FLAG},
        filters::{
            with_bus, with_message_requester, with_requested_messages, with_rest_api_config, with_storage, with_tangle,
        },
        permission::has_permission,
        rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::{body::SuccessBody, responses::WhiteFlagResponse},
};

use bee_ledger::workers::consensus::{self, WhiteFlagMetadata};
use bee_message::{milestone::MilestoneIndex, MessageId};
use bee_protocol::workers::{event::MessageSolidified, request_message, MessageRequesterWorker, RequestedMessages};
use bee_runtime::{event::Bus, resource::ResourceHandle};
use bee_tangle::Tangle;

use futures::channel::oneshot;
use serde_json::Value as JsonValue;
use tokio::time::timeout;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::{
    any::TypeId,
    collections::HashSet,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("whiteflag")).and(warp::path::end())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    storage: ResourceHandle<B>,
    tangle: ResourceHandle<Tangle<B>>,
    bus: ResourceHandle<Bus<'static>>,
    message_requester: MessageRequesterWorker,
    requested_messages: ResourceHandle<RequestedMessages>,
    rest_api_config: RestApiConfig,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::post())
        .and(has_permission(ROUTE_WHITE_FLAG, public_routes, allowed_ips))
        .and(warp::body::json())
        .and(with_storage(storage))
        .and(with_tangle(tangle))
        .and(with_bus(bus))
        .and(with_message_requester(message_requester))
        .and(with_requested_messages(requested_messages))
        .and(with_rest_api_config(rest_api_config))
        .and_then(white_flag)
        .boxed()
}

pub(crate) async fn white_flag<B: StorageBackend>(
    body: JsonValue,
    storage: ResourceHandle<B>,
    tangle: ResourceHandle<Tangle<B>>,
    bus: ResourceHandle<Bus<'static>>,
    message_requester: MessageRequesterWorker,
    requested_messages: ResourceHandle<RequestedMessages>,
    rest_api_config: RestApiConfig,
) -> Result<impl Reply, Rejection> {
    let index_json = &body["index"];
    let parents_json = &body["parentMessageIds"];

    let index = if index_json.is_null() {
        return Err(reject::custom(CustomRejection::BadRequest(
            "Invalid index: expected a MilestoneIndex".to_string(),
        )));
    } else {
        MilestoneIndex(index_json.as_u64().ok_or_else(|| {
            reject::custom(CustomRejection::BadRequest(
                "Invalid index: expected a MilestoneIndex".to_string(),
            ))
        })? as u32)
    };

    let parents = if parents_json.is_null() {
        return Err(reject::custom(CustomRejection::BadRequest(
            "Invalid parents: expected an array of MessageId".to_string(),
        )));
    } else {
        let array = parents_json.as_array().ok_or_else(|| {
            reject::custom(CustomRejection::BadRequest(
                "Invalid parents: expected an array of MessageId".to_string(),
            ))
        })?;
        let mut message_ids = Vec::new();
        for s in array {
            let message_id = s
                .as_str()
                .ok_or_else(|| {
                    reject::custom(CustomRejection::BadRequest(
                        "Invalid parents: expected an array of MessageId".to_string(),
                    ))
                })?
                .parse::<MessageId>()
                .map_err(|_| {
                    reject::custom(CustomRejection::BadRequest(
                        "Invalid parents: expected an array of MessageId".to_string(),
                    ))
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
    bus.add_listener::<Static, _, _>(move |event: &MessageSolidified| {
        if let Ok(mut to_solidify) = task_to_solidify.lock() {
            if to_solidify.remove(&event.message_id) && to_solidify.is_empty() {
                if let Ok(mut sender) = task_sender.lock() {
                    sender.take().map(|s| s.send(()));
                }
            }
        }
    });

    for parent in parents.iter() {
        if tangle.is_solid_message(parent).await {
            if let Ok(mut to_solidify) = to_solidify.lock() {
                to_solidify.remove(parent);
            }
        } else {
            request_message(&*tangle, &message_requester, &*requested_messages, *parent, index).await;
        }
    }

    if let Ok(to_solidify) = to_solidify.lock() {
        if to_solidify.is_empty() {
            if let Ok(mut sender) = sender.lock() {
                sender.take().map(|s| s.send(()));
            }
        }
    }

    let mut metadata = WhiteFlagMetadata::new(index);

    // Wait for either all parents to get solid or the timeout to expire.
    let response = match timeout(
        Duration::from_secs(rest_api_config.white_flag_solidification_timeout()),
        receiver,
    )
    .await
    {
        Ok(_) => {
            // Did not timeout, parents are solid and white flag can happen.
            consensus::white_flag::<B>(&tangle, &storage, &parents, &mut metadata)
                .await
                .map_err(|e| reject::custom(CustomRejection::BadRequest(e.to_string())))?;

            Ok(warp::reply::json(&SuccessBody::new(WhiteFlagResponse {
                merkle_tree_hash: hex::encode(metadata.merkle_proof()),
            })))
        }
        Err(_) => {
            // Did timeout, parents are not solid and white flag can not happen.
            Err(reject::custom(CustomRejection::ServiceUnavailable(
                "parents not solid".to_string(),
            )))
        }
    };

    // Stop listening to the solidification event.
    bus.remove_listeners_by_id(TypeId::of::<Static>());

    response
}
