// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::{BadRequest, NotFound, ServiceUnavailable},
    storage::Backend,
    types::{responses::*, *},
    NetworkId,
};
use bee_common::{node::ResHandle, packable::Packable};
use bee_ledger::spent::Spent;
use bee_message::{payload::milestone::MilestoneEssence, prelude::*};
use bee_protocol::{tangle::MsTangle, MessageSubmitterError, MessageSubmitterWorkerEvent, MilestoneIndex};
use bee_storage::access::Fetch;
use blake2::{Blake2s, VarBlake2b};

use futures::channel::oneshot;
use std::{
    convert::{Infallible, TryFrom, TryInto},
    iter::FromIterator,
    ops::Deref,
    time::{SystemTime, UNIX_EPOCH},
};
use warp::{
    http::{Response, StatusCode},
    reject, Buf, Rejection, Reply,
};

use flume::Sender;
use log::error;
use serde_json::Value as JsonValue;

pub(crate) async fn get_health<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> Result<impl Reply, Infallible> {
    if is_healthy(tangle).await {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::SERVICE_UNAVAILABLE)
    }
}

pub(crate) async fn get_info<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
    network_id: NetworkId,
) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&DataResponse::new(GetInfoResponse {
        name: String::from("Bee"),
        version: String::from(env!("CARGO_PKG_VERSION")),
        is_healthy: is_healthy(tangle.clone()).await,
        network_id: network_id.0,
        latest_milestone_index: *tangle.get_latest_milestone_index(),
        solid_milestone_index: *tangle.get_latest_milestone_index(),
        pruning_index: *tangle.get_pruning_index(),
        features: Vec::new(), // TODO: look up features
    })))
}

pub(crate) async fn get_tips<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> Result<impl Reply, Rejection> {
    match tangle.get_messages_to_approve().await {
        Some(tips) => Ok(warp::reply::json(&DataResponse::new(GetTipsResponse {
            tip_1_message_id: tips.0.to_string(),
            tip_2_message_id: tips.1.to_string(),
        }))),
        None => Err(reject::custom(ServiceUnavailable("tip pool is empty".to_string()))),
    }
}

pub(crate) async fn submit_json_message<B: Backend>(
    value: JsonValue,
    tangle: ResHandle<MsTangle<B>>,
    message_submitter: Sender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
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
                    "can not parse network id: expected an u64-string".to_string(),
                )))?
                .parse::<u64>()
                .map_err(|_| {
                    reject::custom(BadRequest(
                        "can not parse network id: expected an u64-string".to_string(),
                    ))
                })?
        }
    };

    let (parent_1_message_id, parent_2_message_id) = {
        let parent_1_v = &value["parent1MessageId"];
        let parent_2_v = &value["parent2MessageId"];

        if is_set(parent_1_v) && is_set(parent_2_v) {
            // if both parents are set
            let parent1 = parent_1_v
                .as_str()
                .ok_or(reject::custom(BadRequest(
                    "can not parse parent 1: expected a hex-string".to_string(),
                )))?
                .parse::<MessageId>()
                .map_err(|_| reject::custom(BadRequest("can not parse parent 1: not a message id".to_string())))?;
            let parent2 = parent_2_v
                .as_str()
                .ok_or(reject::custom(BadRequest(
                    "can not parse parent 2: expected a hex-string".to_string(),
                )))?
                .parse::<MessageId>()
                .map_err(|_| reject::custom(BadRequest("can not parse parent 2: not a message id".to_string())))?;
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
                    .ok_or(reject::custom(BadRequest(
                        "can not parse parent 1: expected a hex-string".to_string(),
                    )))?
                    .parse::<MessageId>()
                    .map_err(|_| reject::custom(BadRequest("can not parse parent 1: not a message id".to_string())))?
            } else {
                parent_2_v
                    .as_str()
                    .ok_or(reject::custom(BadRequest(
                        "can not parse parent 2: expected a hex-string".to_string(),
                    )))?
                    .parse::<MessageId>()
                    .map_err(|_| reject::custom(BadRequest("can not parse parent 2: not a message id".to_string())))?
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
            return Err(reject::custom(ServiceUnavailable(
                "can not auto-fill nonce: remote proof-of-work not supported at the moment".to_string(),
            )));
        } else {
            nonce_v
                .as_str()
                .ok_or(reject::custom(BadRequest(
                    "can not parse nonce: expected an u64-string".to_string(),
                )))?
                .parse::<u64>()
                .map_err(|_| reject::custom(BadRequest("can not parse nonce: expected an u64-string".to_string())))?
        }
    };

    let message = {
        let mut builder = Message::builder()
            .with_network_id(network_id)
            .with_parent1(parent_1_message_id)
            .with_parent2(parent_2_message_id)
            .with_nonce(nonce);
        if let Some(payload) = payload {
            builder = builder.with_payload(payload)
        }
        builder
            .finish()
            .map_err(|e| reject::custom(BadRequest(e.to_string())))?
    };

    let message_id = submit_message(message, tangle, message_submitter).await?;

    Ok(warp::reply::json(&DataResponse::new(PostMessageResponse {
        message_id: message_id.to_string(),
    })))
}

pub(crate) async fn submit_raw_message<B: Backend>(
    buf: warp::hyper::body::Bytes,
    tangle: ResHandle<MsTangle<B>>,
    message_submitter: flume::Sender<MessageSubmitterWorkerEvent>,
) -> Result<impl Reply, Rejection> {
    let message = Message::unpack(&mut buf.bytes()).map_err(|e| reject::custom(BadRequest(e.to_string())))?;
    let message_id = submit_message(message, tangle, message_submitter).await?;
    Ok(warp::reply::json(&DataResponse::new(PostMessageResponse {
        message_id: message_id.to_string(),
    })))
}

pub(crate) async fn get_message_by_index<B: Backend>(
    index: String,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    let hashed_index = {
        use digest::Digest;
        let mut hasher = Blake2s::new();
        hasher.update(index.as_bytes());
        // `Blake2s` output is `HASHED_INDEX_LENGTH` bytes long.
        HashedIndex::new(hasher.finalize_reset().as_slice().try_into().unwrap())
    };

    let mut fetched = match Fetch::<HashedIndex, Vec<MessageId>>::fetch(storage.deref(), &hashed_index)
        .await
        .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?
    {
        Some(ids) => ids,
        None => vec![],
    };

    let count = fetched.len();
    let max_results = 1000;
    fetched.truncate(max_results);

    Ok(warp::reply::json(&DataResponse::new(GetMessagesByIndexResponse {
        index,
        max_results,
        count,
        message_ids: fetched.iter().map(|id| id.to_string()).collect(),
    })))
}

pub(crate) async fn get_message_by_message_id<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&message_id).await {
        Some(message) => Ok(warp::reply::json(&DataResponse::new(GetMessageResponse(
            MessageDto::try_from(&*message).map_err(|e| reject::custom(BadRequest(e.to_string())))?,
        )))),
        None => Err(reject::custom(NotFound("can not find message".to_string()))),
    }
}

pub(crate) async fn get_message_metadata<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    if !tangle.is_synced() {
        return Err(reject::custom(ServiceUnavailable("tangle not synced".to_string())));
    }

    let ytrsi_delta = 8;
    let otrsi_delta = 13;
    let below_max_depth = 15;

    match tangle.get_metadata(&message_id) {
        Some(metadata) => {
            match tangle.get(&message_id).await {
                Some(message) => {
                    let res = {
                        // in case the message is referenced by a milestone
                        if let Some(milestone) = metadata.cone_index() {
                            GetMessageMetadataResponse {
                                message_id: message_id.to_string(),
                                parent_1_message_id: message.parent1().to_string(),
                                parent_2_message_id: message.parent2().to_string(),
                                is_solid: metadata.flags().is_solid(),
                                referenced_by_milestone_index: Some(*milestone),
                                ledger_inclusion_state: Some(
                                    if let Some(Payload::Transaction(_)) = message.payload() {
                                        if metadata.flags().is_conflicting() {
                                            LedgerInclusionStateDto::Conflicting
                                        } else {
                                            LedgerInclusionStateDto::Included
                                        }
                                    } else {
                                        LedgerInclusionStateDto::NoTransaction
                                    },
                                ),
                                should_promote: None,
                                should_reattach: None,
                            }
                        } else {
                            // in case the message is not referenced by a milestone, but solid
                            if metadata.flags().is_solid() {
                                let mut should_promote = false;
                                let mut should_reattach = false;
                                let lsmi = *tangle.get_latest_solid_milestone_index();

                                if (lsmi - otrsi_delta) > below_max_depth {
                                    should_promote = false;
                                    should_reattach = true;
                                } else if (lsmi - ytrsi_delta) > ytrsi_delta {
                                    should_promote = true;
                                    should_reattach = false;
                                } else if (lsmi - otrsi_delta) > otrsi_delta {
                                    should_promote = true;
                                    should_reattach = false;
                                }

                                GetMessageMetadataResponse {
                                    message_id: message_id.to_string(),
                                    parent_1_message_id: message.parent1().to_string(),
                                    parent_2_message_id: message.parent2().to_string(),
                                    is_solid: true,
                                    referenced_by_milestone_index: None,
                                    ledger_inclusion_state: None,
                                    should_promote: Some(should_promote),
                                    should_reattach: Some(should_reattach),
                                }
                            } else {
                                // in case the message is not referenced by a milestone, not solid,
                                GetMessageMetadataResponse {
                                    message_id: message_id.to_string(),
                                    parent_1_message_id: message.parent1().to_string(),
                                    parent_2_message_id: message.parent2().to_string(),
                                    is_solid: false,
                                    referenced_by_milestone_index: None,
                                    ledger_inclusion_state: None,
                                    should_promote: Some(true),
                                    should_reattach: Some(false),
                                }
                            }
                        }
                    };

                    Ok(warp::reply::json(&DataResponse::new(res)))
                }
                None => Err(reject::custom(NotFound("can not find data".to_string()))),
            }
        }
        None => Err(reject::custom(NotFound("can not find data".to_string()))),
    }
}

pub async fn get_raw_message<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&message_id).await {
        Some(message) => Ok(Response::builder()
            .header("Content-Type", "application/octet-stream")
            .body(message.pack_new())),
        None => Err(reject::custom(NotFound("can not find message".to_string()))),
    }
}

pub async fn get_children_by_message_id<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    let mut children = Vec::from_iter(tangle.get_children(&message_id));
    let count = children.len();
    let max_results = 1000;
    children.truncate(max_results);
    Ok(warp::reply::json(&DataResponse::new(GetChildrenResponse {
        message_id: message_id.to_string(),
        max_results,
        count,
        children_message_ids: children.iter().map(|id| id.to_string()).collect(),
    })))
}

pub(crate) async fn get_milestone_by_milestone_index<B: Backend>(
    milestone_index: MilestoneIndex,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get_milestone_message_id(milestone_index) {
        Some(message_id) => match tangle.get_metadata(&message_id) {
            Some(metadata) => Ok(warp::reply::json(&DataResponse::new(GetMilestoneResponse {
                milestone_index: *milestone_index,
                message_id: message_id.to_string(),
                timestamp: metadata.arrival_timestamp(),
            }))),
            None => Err(reject::custom(NotFound(
                "can not find metadata for milestone".to_string(),
            ))),
        },
        None => Err(reject::custom(NotFound("can not find milestone".to_string()))),
    }
}

pub(crate) async fn get_output_by_output_id<B: Backend>(
    output_id: OutputId,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    let output = Fetch::<OutputId, bee_ledger::output::Output>::fetch(storage.deref(), &output_id)
        .await
        .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?;
    let is_spent = Fetch::<OutputId, Spent>::fetch(storage.deref(), &output_id)
        .await
        .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?;
    match output {
        Some(output) => Ok(warp::reply::json(&DataResponse::new(GetOutputByOutputIdResponse {
            message_id: output.message_id().to_string(),
            transaction_id: output_id.transaction_id().to_string(),
            output_index: output_id.index(),
            is_spent: is_spent.is_some(),
            output: output
                .inner()
                .try_into()
                .map_err(|e: &str| reject::custom(BadRequest(e.to_string())))?,
        }))),
        None => Err(reject::custom(NotFound("can not find output".to_string()))),
    }
}

pub(crate) async fn get_balance_for_address<B: Backend>(
    addr: Ed25519Address,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    match Fetch::<Ed25519Address, Vec<OutputId>>::fetch(storage.deref(), &addr)
        .await
        .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?
    {
        Some(mut ids) => {
            let max_results = 1000;
            let count = ids.len();
            ids.truncate(max_results);
            let mut balance = 0;
            for id in ids {
                if let Some(output) = Fetch::<OutputId, bee_ledger::output::Output>::fetch(storage.deref(), &id)
                    .await
                    .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?
                {
                    if let None = Fetch::<OutputId, Spent>::fetch(storage.deref(), &id)
                        .await
                        .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?
                    {
                        match output.inner() {
                            Output::SignatureLockedSingle(o) => balance += o.amount().get() as u64,
                            _ => return Err(reject::custom(ServiceUnavailable("output type not supported".to_string())))
                        }
                    }
                }
            }
            Ok(warp::reply::json(&DataResponse::new(GetBalanceForAddressResponse {
                address: addr.to_string(),
                max_results,
                count,
                balance,
            })))
        }
        None => Err(reject::custom(NotFound("can not find output ids".to_string()))),
    }
}

pub(crate) async fn get_outputs_for_address<B: Backend>(
    addr: Ed25519Address,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    let mut fetched = match Fetch::<Ed25519Address, Vec<OutputId>>::fetch(storage.deref(), &addr)
        .await
        .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?
    {
        Some(ids) => ids,
        None => vec![],
    };

    let count = fetched.len();
    let max_results = 1000;
    fetched.truncate(max_results);

    Ok(warp::reply::json(&DataResponse::new(GetOutputsForAddressResponse {
        address: addr.to_string(),
        max_results,
        count,
        output_ids: fetched.iter().map(|id| id.to_string()).collect(),
    })))
}

async fn is_healthy<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> bool {
    if !tangle.is_synced() {
        return false;
    }

    // TODO: check if number of peers != 0 else return false

    match tangle.get_milestone_message_id(tangle.get_latest_milestone_index()) {
        Some(milestone_message_id) => match tangle.get_metadata(&milestone_message_id) {
            Some(metadata) => {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Clock may have gone backwards")
                    .as_millis() as u64;
                let latest_milestone_arrival_timestamp = metadata.arrival_timestamp();
                if current_time - latest_milestone_arrival_timestamp > 5 * 60 * 60000 {
                    return false;
                }
            }
            None => return false,
        },
        None => return false,
    }

    true
}

async fn submit_message<B: Backend>(
    message: Message,
    tangle: ResHandle<MsTangle<B>>,
    message_submitter: Sender<MessageSubmitterWorkerEvent>,
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
            error!("can not submit message: {:?}", e);
            reject::custom(ServiceUnavailable("can not submit message".to_string()))
        })?;

    let result = waiter.await.map_err(|_| {
        // TODO: report back from HasherWorker and replace following line with:
        // error!("can not submit message: {:?}",e);
        reject::custom(BadRequest(
            "invalid message, already received in a previous request".to_string(),
        ))
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

pub mod tests {

    use super::*;

    #[allow(dead_code)]
    pub fn message_without_payload() -> Message {
        Message::builder()
            .with_network_id(1)
            .with_parent1(MessageId::new([
                0xF5, 0x32, 0xA5, 0x35, 0x45, 0x10, 0x32, 0x76, 0xB4, 0x68, 0x76, 0xC4, 0x73, 0x84, 0x6D, 0x98, 0x64,
                0x8E, 0xE4, 0x18, 0x46, 0x8B, 0xCE, 0x76, 0xDF, 0x48, 0x68, 0x64, 0x8D, 0xD7, 0x3E, 0x5D,
            ]))
            .with_parent2(MessageId::new([
                0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
            ]))
            .finish()
            .unwrap()
    }

    #[allow(dead_code)]
    pub fn message_with_indexation_payload() -> Message {
        Message::builder()
            .with_network_id(1)
            .with_parent1(MessageId::new([
                0xF5, 0x32, 0xA5, 0x35, 0x45, 0x10, 0x32, 0x76, 0xB4, 0x68, 0x76, 0xC4, 0x73, 0x84, 0x6D, 0x98, 0x64,
                0x8E, 0xE4, 0x18, 0x46, 0x8B, 0xCE, 0x76, 0xDF, 0x48, 0x68, 0x64, 0x8D, 0xD7, 0x3E, 0x5D,
            ]))
            .with_parent2(MessageId::new([
                0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
            ]))
            .with_payload(Payload::Indexation(Box::new(
                Indexation::new(
                    "MYINDEX".to_owned(),
                    &[0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f, 0x74, 0x61],
                )
                .unwrap(),
            )))
            .finish()
            .unwrap()
    }

    #[allow(dead_code)]
    pub fn message_with_milestone_payload() -> Message {
        Message::builder()
            .with_network_id(1)
            .with_parent1(MessageId::new([
                0xF5, 0x32, 0xA5, 0x35, 0x45, 0x10, 0x32, 0x76, 0xB4, 0x68, 0x76, 0xC4, 0x73, 0x84, 0x6D, 0x98, 0x64,
                0x8E, 0xE4, 0x18, 0x46, 0x8B, 0xCE, 0x76, 0xDF, 0x48, 0x68, 0x64, 0x8D, 0xD7, 0x3E, 0x5D,
            ]))
            .with_parent2(MessageId::new([
                0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
            ]))
            .with_payload(Payload::Milestone(Box::new(Milestone::new(
                MilestoneEssence::new(
                    1633,
                    1604072711,
                    MessageId::new([
                        0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                        0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
                    ]),
                    MessageId::new([
                        0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                        0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
                    ]),
                    hex::decode("786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce").unwrap().into_boxed_slice(),

                    vec![[0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f,0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f,0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f,0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f,]],
                ),
                vec![
                    hex::decode("a3676743c128a78323598965ef89c43ab412e207083feb80fbb3e3a4327aa4bb161f7be427641a21b23af9a58c5a0efdd36f26b2af893e7ad899b76f19cc410d").unwrap().into_boxed_slice(),
             ]
            )))).finish()
            .unwrap()
    }
}
