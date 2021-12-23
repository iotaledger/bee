// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_MESSAGE_METADATA, filters::with_tangle, path_params::message_id, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend, CONFIRMED_THRESHOLD,
    },
    types::{body::SuccessBody, dtos::LedgerInclusionStateDto, responses::MessageMetadataResponse},
};

use bee_message::{payload::Payload, MessageId};
use bee_runtime::resource::ResourceHandle;
use bee_tangle::{ConflictReason, Tangle};

use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (MessageId,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("messages"))
        .and(message_id())
        .and(warp::path("metadata"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGE_METADATA, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(message_metadata)
        .boxed()
}

pub(crate) async fn message_metadata<B: StorageBackend>(
    message_id: MessageId,
    tangle: ResourceHandle<Tangle<B>>,
) -> Result<impl Reply, Rejection> {
    if !tangle.is_confirmed_threshold(CONFIRMED_THRESHOLD) {
        return Err(reject::custom(CustomRejection::ServiceUnavailable(
            "the node is not synchronized".to_string(),
        )));
    }

    match tangle.get(&message_id).await.map(|m| (*m).clone()) {
        Some(message) => {
            // existing message <=> existing metadata, therefore unwrap() is safe
            let metadata = tangle.get_metadata(&message_id).await.unwrap();

            // TODO: access constants from URTS
            let ymrsi_delta = 8;
            let omrsi_delta = 13;
            let below_max_depth = 15;

            let (
                is_solid,
                referenced_by_milestone_index,
                milestone_index,
                ledger_inclusion_state,
                conflict_reason,
                should_promote,
                should_reattach,
            ) = {
                let is_solid;
                let referenced_by_milestone_index;
                let milestone_index;
                let ledger_inclusion_state;
                let conflict_reason;
                let should_promote;
                let should_reattach;

                if let Some(milestone) = metadata.milestone_index() {
                    // message is referenced by a milestone
                    is_solid = true;
                    referenced_by_milestone_index = Some(*milestone);

                    if metadata.flags().is_milestone() {
                        milestone_index = Some(*milestone);
                    } else {
                        milestone_index = None;
                    }

                    ledger_inclusion_state = Some(if let Some(Payload::Transaction(_)) = message.payload() {
                        if metadata.conflict() != ConflictReason::None {
                            conflict_reason = Some(metadata.conflict());
                            LedgerInclusionStateDto::Conflicting
                        } else {
                            conflict_reason = None;
                            // maybe not checked by the ledger yet, but still
                            // returning "included". should
                            // `metadata.flags().is_conflicting` return an Option
                            // instead?
                            LedgerInclusionStateDto::Included
                        }
                    } else {
                        conflict_reason = None;
                        LedgerInclusionStateDto::NoTransaction
                    });
                    should_reattach = None;
                    should_promote = None;
                } else if metadata.flags().is_solid() {
                    // message is not referenced by a milestone but solid
                    is_solid = true;
                    referenced_by_milestone_index = None;
                    milestone_index = None;
                    ledger_inclusion_state = None;
                    conflict_reason = None;

                    let cmi = *tangle.get_confirmed_milestone_index();
                    // unwrap() of OMRSI/YMRSI is safe since message is solid
                    if (cmi - *metadata.omrsi().unwrap().index()) > below_max_depth {
                        should_promote = Some(false);
                        should_reattach = Some(true);
                    } else if (cmi - *metadata.ymrsi().unwrap().index()) > ymrsi_delta
                        || (cmi - *metadata.omrsi().unwrap().index()) > omrsi_delta
                    {
                        should_promote = Some(true);
                        should_reattach = Some(false);
                    } else {
                        should_promote = Some(false);
                        should_reattach = Some(false);
                    };
                } else {
                    // the message is not referenced by a milestone and not solid
                    is_solid = false;
                    referenced_by_milestone_index = None;
                    milestone_index = None;
                    ledger_inclusion_state = None;
                    conflict_reason = None;
                    should_reattach = Some(true);
                    should_promote = Some(false);
                }

                (
                    is_solid,
                    referenced_by_milestone_index,
                    milestone_index,
                    ledger_inclusion_state,
                    conflict_reason,
                    should_reattach,
                    should_promote,
                )
            };

            Ok(warp::reply::json(&SuccessBody::new(MessageMetadataResponse {
                message_id: message_id.to_string(),
                parent_message_ids: message.parents().iter().map(|id| id.to_string()).collect(),
                is_solid,
                referenced_by_milestone_index,
                milestone_index,
                ledger_inclusion_state,
                conflict_reason: conflict_reason.map(|c| c as u8),
                should_promote,
                should_reattach,
            })))
        }
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find message".to_string(),
        ))),
    }
}
