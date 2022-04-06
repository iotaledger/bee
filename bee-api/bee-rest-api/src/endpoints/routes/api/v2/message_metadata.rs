// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_message::{payload::Payload, semantic::ConflictReason, MessageId};

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode, CONFIRMED_THRESHOLD},
    types::{dtos::LedgerInclusionStateDto, responses::MessageMetadataResponse},
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/messages/:message_id/metadata", get(message_metadata::<B>))
}

pub(crate) async fn message_metadata<B: StorageBackend>(
    Path(message_id): Path<MessageId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    if !args.tangle.is_confirmed_threshold(CONFIRMED_THRESHOLD) {
        return Err(ApiError::ServiceUnavailable("the node is not synchronized".to_string()));
    }

    match args.tangle.get(&message_id).await.map(|m| (*m).clone()) {
        Some(message) => {
            // Panic: existing message <=> existing metadata => unwrap() is safe.
            let metadata = args.tangle.get_metadata(&message_id).await.unwrap();

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

                    let cmi = *args.tangle.get_confirmed_milestone_index();
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

            Ok(Json(MessageMetadataResponse {
                message_id: message_id.to_string(),
                parent_message_ids: message.parents().iter().map(MessageId::to_string).collect(),
                is_solid,
                referenced_by_milestone_index,
                milestone_index,
                ledger_inclusion_state,
                conflict_reason: conflict_reason.map(|c| c as u8),
                should_promote,
                should_reattach,
            }))
        }
        None => Err(ApiError::NotFound("cannot find message".to_string())),
    }
}
