// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::{BadRequest, NotFound, ServiceUnavailable},
    storage::Backend,
    types::{DataResponse, GetInfoResponse, GetMilestoneResponse, GetTipsResponse, *},
    NetworkId,
};

use bee_common::{node::ResHandle, packable::Packable};
use bee_ledger::spent::Spent;
use bee_message::{payload::milestone::MilestoneEssence, prelude::*};
use bee_protocol::{tangle::MsTangle, MessageSubmitterError, MessageSubmitterWorkerEvent, MilestoneIndex};
use bee_storage::access::Fetch;

use blake2::Blake2s;
use digest::Digest;
use futures::channel::oneshot;
use warp::{
    http::{Response, StatusCode},
    reject, Rejection, Reply,
};

use std::{
    convert::{Infallible, TryInto},
    iter::FromIterator,
    ops::Deref,
    time::{SystemTime, UNIX_EPOCH},
};

async fn is_healthy<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> bool {
    let mut is_healthy = true;

    if !tangle.is_synced() {
        is_healthy = false;
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
                    is_healthy = false;
                }
            }
            None => is_healthy = false,
        },
        None => is_healthy = false,
    }

    is_healthy
}

pub async fn get_health<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> Result<impl Reply, Infallible> {
    if is_healthy(tangle).await {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::SERVICE_UNAVAILABLE)
    }
}

pub async fn get_info<B: Backend>(
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
        features: Vec::new(), // TODO: replace placeholder
    })))
}

pub async fn get_tips<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> Result<impl Reply, Rejection> {
    match tangle.get_messages_to_approve().await {
        Some(tips) => Ok(warp::reply::json(&DataResponse::new(GetTipsResponse {
            tip_1_message_id: tips.0.to_string(),
            tip_2_message_id: tips.1.to_string(),
        }))),
        None => Err(reject::custom(ServiceUnavailable("no tips available"))),
    }
}

pub async fn post_json_message<B: Backend>(
    input: serde_json::Value,
    _message_submitter: flume::Sender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    let _network_id = if input["network_id"].is_null() {
        network_id.1
    } else {
        input["network_id"]
            .as_str()
            .ok_or(reject::custom(BadRequest("invalid network id: not a string")))?
            .parse::<u64>()
            .map_err(|_| reject::custom(BadRequest("invalid network id: can not parse u64 from string")))?
    };

    let (_parent_1_message_id, _parent_2_message_id) = {
        let parent_1 = &input["parent1MessageId"];
        let parent_2 = &input["parent2MessageId"];
        if parent_1.is_null() && parent_2.is_null() {
            tangle
                .get_messages_to_approve()
                .await
                .ok_or(reject::custom(ServiceUnavailable("no tips, try again later")))?
        } else {
            if !parent_1.is_null() && !parent_2.is_null() {
                (
                    parent_1
                        .as_str()
                        .ok_or(reject::custom(BadRequest("invalid parent: not a string")))?
                        .parse::<MessageId>()
                        .map_err(|_| reject::custom(BadRequest("invalid parent provided")))?,
                    parent_2
                        .as_str()
                        .ok_or(reject::custom(BadRequest("invalid parent: not a string")))?
                        .parse::<MessageId>()
                        .map_err(|_| reject::custom(BadRequest("invalid parent provided")))?,
                )
            } else {
                return Err(reject::custom(BadRequest("one parent is missing")));
            }
        }
    };

    Ok(StatusCode::OK)
}

pub async fn post_raw_message(
    buf: warp::hyper::body::Bytes,
    message_submitter: flume::Sender<MessageSubmitterWorkerEvent>,
) -> Result<impl Reply, Rejection> {
    let min_msg_size: usize = 77;
    let max_msg_size: usize = 1024;

    if buf.len() < min_msg_size || buf.len() > max_msg_size {
        return Err(reject::custom(BadRequest("invalid message length")));
    }

    let (notifier_tx, notifier_rx) = oneshot::channel::<Result<MessageId, MessageSubmitterError>>();

    message_submitter
        .send(MessageSubmitterWorkerEvent {
            buf: buf.to_vec(),
            notifier: notifier_tx,
        })
        .map_err(|_| reject::custom(ServiceUnavailable("service unavailable")))?;

    match notifier_rx
        .await
        .map_err(|_| reject::custom(ServiceUnavailable("service unavailable")))?
    {
        Ok(message_id) => Ok(warp::reply::json(&DataResponse::new(PostMessageResponse {
            message_id: message_id.to_string(),
        }))),
        Err(_err) => Err(reject::custom(BadRequest("invalid message"))),
    }
}

pub async fn get_message_by_index<B: Backend>(index: String, storage: ResHandle<B>) -> Result<impl Reply, Rejection> {
    let mut hasher = Blake2s::new();
    hasher.update(index.as_bytes());
    let hashed_index = HashedIndex::new(hasher.finalize_reset().as_slice().try_into().unwrap());

    let max_results = 1000;
    match storage.deref().fetch(&hashed_index).await {
        Ok(ret) => match ret {
            Some(mut fetched) => {
                let count = fetched.len();
                fetched.truncate(max_results);
                Ok(warp::reply::json(&DataResponse::new(GetMessagesByIndexResponse {
                    index,
                    max_results,
                    count,
                    message_ids: fetched.iter().map(|id| id.to_string()).collect(),
                })))
            }
            None => Ok(warp::reply::json(&DataResponse::new(GetMessagesByIndexResponse {
                index,
                max_results,
                count: 0,
                message_ids: vec![],
            }))),
        },
        Err(_) => Err(reject::custom(ServiceUnavailable("service unavailable"))),
    }
}

pub async fn get_message_by_message_id<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&message_id).await {
        Some(message) => {
            let network_id = message.network_id().to_string();
            let parent_1_message_id = message.parent1().to_string();
            let parent_2_message_id = message.parent2().to_string();
            let payload = {
                match message.payload() {
                    Some(Payload::Transaction(t)) => Some(PayloadDto::Transaction(TransactionPayloadDto {
                        kind: 0,
                        essence: TransactionEssenceDto {
                            kind: 0,
                            inputs: t
                                .essence()
                                .inputs()
                                .iter()
                                .map(|input| match input {
                                    Input::UTXO(input) => UtxoInputDto {
                                        kind: 0,
                                        transaction_id: input.output_id().transaction_id().to_string(),
                                        transaction_output_index: input.output_id().index(),
                                    },
                                    _ => unimplemented!(),
                                })
                                .collect(),
                            outputs: t
                                .essence()
                                .outputs()
                                .iter()
                                .map(|output| match output {
                                    Output::SignatureLockedSingle(output) => {
                                        OutputDto::SignatureLockedSingle(SignatureLockedSingleOutputDto {
                                            kind: 0,
                                            address: match output.address() {
                                                Address::Ed25519(ed) => Ed25519AddressDto {
                                                    kind: 1,
                                                    address: ed.to_bech32(),
                                                },
                                                _ => unimplemented!(),
                                            },
                                            amount: output.amount().to_string(),
                                        })
                                    }
                                    _ => unimplemented!(),
                                })
                                .collect(),
                            payload: match t.essence().payload() {
                                Some(Payload::Indexation(i)) => Some(IndexationPayloadDto {
                                    kind: 2,
                                    index: i.index().to_owned(),
                                    data: hex::encode(i.data()),
                                }),
                                Some(_) => unreachable!(),
                                None => None,
                            },
                        },
                        unlock_blocks: t
                            .unlock_blocks()
                            .iter()
                            .map(|unlock_block| match unlock_block {
                                UnlockBlock::Signature(s) => match s {
                                    SignatureUnlock::Ed25519(ed) => {
                                        UnlockBlockDto::Signature(SignatureUnlockBlockDto {
                                            kind: 1,
                                            signature: crate::types::Ed25519SignatureDto {
                                                kind: 0,
                                                public_key: hex::encode(ed.public_key()),
                                                signature: hex::encode(ed.signature()),
                                            },
                                        })
                                    }
                                    _ => unimplemented!(),
                                },
                                UnlockBlock::Reference(r) => UnlockBlockDto::Reference(ReferenceUnlockBlockDto {
                                    kind: 1,
                                    reference: r.index(),
                                }),
                                _ => unimplemented!(),
                            })
                            .collect(),
                    })),
                    Some(Payload::Milestone(m)) => Some(PayloadDto::Milestone(MilestonePayloadDto {
                        kind: 1,
                        index: m.essence().index(),
                        timestamp: m.essence().timestamp(),
                        inclusion_merkle_proof: hex::encode(m.essence().merkle_proof()),
                        signatures: m.signatures().iter().map(|sig| hex::encode(sig)).collect(),
                    })),
                    Some(Payload::Indexation(i)) => Some(PayloadDto::Indexation(IndexationPayloadDto {
                        kind: 2,
                        index: i.index().to_owned(),
                        data: hex::encode(i.data()),
                    })),
                    Some(_) => unimplemented!(),
                    None => None,
                }
            };
            let nonce = message.nonce().to_string();

            Ok(warp::reply::json(&DataResponse::new(GetMessageResponse(MessageDto {
                network_id,
                parent_1_message_id,
                parent_2_message_id,
                payload,
                nonce,
            }))))
        }
        None => Err(reject::custom(NotFound("can not find data"))),
    }
}

pub async fn get_message_metadata<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    if !tangle.is_synced() {
        return Err(reject::custom(ServiceUnavailable(
            "service unavailable: the tangle is not synced",
        )));
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
                None => Err(reject::custom(NotFound("can not find data"))),
            }
        }
        None => Err(reject::custom(NotFound("can not find data"))),
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
        None => Err(reject::custom(NotFound("can not find data"))),
    }
}

pub async fn get_children_by_message_id<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    if tangle.contains(&message_id).await {
        let max_results = 1000;
        let mut children = Vec::from_iter(tangle.get_children(&message_id));
        let count = children.len();
        children.truncate(max_results);
        Ok(warp::reply::json(&DataResponse::new(GetChildrenResponse {
            message_id: message_id.to_string(),
            max_results,
            count,
            children_message_ids: children.iter().map(|id| id.to_string()).collect(),
        })))
    } else {
        Err(reject::custom(NotFound("can not find data")))
    }
}

pub async fn get_milestone_by_milestone_index<B: Backend>(
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
            None => Err(reject::custom(NotFound("can not find data"))),
        },
        None => Err(reject::custom(NotFound("can not find data"))),
    }
}

pub async fn get_output_by_output_id<B: Backend>(
    output_id: OutputId,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    let output: Result<Option<bee_ledger::output::Output>, <B as bee_storage::storage::Backend>::Error> =
        storage.fetch(&output_id).await;
    let is_spent: Result<Option<Spent>, <B as bee_storage::storage::Backend>::Error> = storage.fetch(&output_id).await;

    if let (Ok(output), Ok(is_spent)) = (output, is_spent) {
        match output {
            Some(output) => Ok(warp::reply::json(&DataResponse::new(GetOutputByOutputIdResponse {
                message_id: output.message_id().to_string(),
                transaction_id: output_id.transaction_id().to_string(),
                output_index: output_id.index(),
                is_spent: is_spent.is_some(),
                output: match output.inner() {
                    Output::SignatureLockedSingle(output) => {
                        OutputDto::SignatureLockedSingle(SignatureLockedSingleOutputDto {
                            kind: 0,
                            address: Ed25519AddressDto {
                                kind: 1,
                                address: output.address().to_bech32(),
                            },
                            amount: output.amount().to_string(),
                        })
                    }
                    _ => panic!("unexpected signature scheme"),
                },
            }))),
            None => Err(reject::custom(NotFound("can not find data"))),
        }
    } else {
        Err(reject::custom(ServiceUnavailable(
            "service unavailable: can not fetch from storage",
        )))
    }
}

pub async fn get_balance_for_address<B: Backend>(
    addr: Ed25519Address,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    match Fetch::<Ed25519Address, Vec<OutputId>>::fetch(storage.deref(), &addr)
        .await
        .map_err(|_| reject::custom(ServiceUnavailable("service unavailable: can not fetch from storage")))?
    {
        Some(mut ids) => {
            let max_results = 1000;
            let count = ids.len();
            ids.truncate(max_results);
            let mut balance = 0;

            for id in ids {
                if let Some(output) = Fetch::<OutputId, bee_ledger::output::Output>::fetch(storage.deref(), &id)
                    .await
                    .map_err(|_| {
                        reject::custom(ServiceUnavailable("service unavailable: can not fetch from storage"))
                    })?
                {
                    if let None = Fetch::<OutputId, Spent>::fetch(storage.deref(), &id)
                        .await
                        .map_err(|_| {
                            reject::custom(ServiceUnavailable("service unavailable: can not fetch from storage"))
                        })?
                    {
                        match output.inner() {
                            Output::SignatureLockedSingle(o) => balance += o.amount().get() as u32,
                            _ => {}
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
        None => Err(reject::custom(NotFound("can not find data"))),
    }
}

pub async fn get_outputs_for_address<B: Backend>(
    addr: Ed25519Address,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    match Fetch::<Ed25519Address, Vec<OutputId>>::fetch(storage.deref(), &addr).await {
        Ok(res) => match res {
            Some(mut ids) => {
                let max_results = 1000;
                let count = ids.len();
                ids.truncate(max_results);
                Ok(warp::reply::json(&DataResponse::new(GetOutputsForAddressResponse {
                    address: addr.to_string(),
                    max_results,
                    count,
                    output_ids: ids.iter().map(|id| id.to_string()).collect(),
                })))
            }
            None => Err(reject::custom(NotFound("can not find data"))),
        },
        Err(_err) => Err(reject::custom(ServiceUnavailable(
            "service unavailable: can not fetch from storage",
        ))),
    }
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
    pub fn indexation_message() -> Message {
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
    pub fn milestone_message() -> Message {
        Message::builder()
            //.with_network_id(1)
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
                    hex::decode("b3676743c128a78323598965ef89c43ab412e207083feb80fbb3e3a4327aa4bb161f7be427641a21b23af9a58c5a0efdd36f26b2af893e7ad899b76f19cc410d").unwrap().into_boxed_slice(),
                ]
            )))).finish()
            .unwrap()
    }
}
