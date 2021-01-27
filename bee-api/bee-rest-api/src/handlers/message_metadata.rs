// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::NotFound,
    handlers::{BodyInner, SuccessBody},
    storage::StorageBackend,
};

use bee_ledger::conflict::ConflictReason;
use bee_message::prelude::*;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use serde::{Deserialize, Serialize};
use warp::{reject, Rejection, Reply};

pub(crate) async fn message_metadata<B: StorageBackend>(
    message_id: MessageId,
    tangle: ResourceHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    // TODO: response only when the node is synced
    match tangle.get(&message_id).await.map(|m| (*m).clone()) {
        Some(message) => {
            // existing message <=> existing metadata, therefore unwrap() is safe
            let metadata = tangle.get_metadata(&message_id).await.unwrap();

            // TODO: access constants from URTS
            let ytrsi_delta = 8;
            let otrsi_delta = 13;
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

                if let Some(milestone) = metadata.cone_index() {
                    // message is referenced by a milestone
                    is_solid = true;
                    referenced_by_milestone_index = Some(*milestone);

                    if metadata.flags().is_milestone() {
                        milestone_index = Some(*milestone);
                    } else {
                        milestone_index = None;
                    }

                    ledger_inclusion_state = Some(if let Some(Payload::Transaction(_)) = message.payload() {
                        if metadata.conflict() != ConflictReason::None as u8 {
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

                    let lsmi = *tangle.get_latest_solid_milestone_index();
                    // unwrap() of OTRSI/YTRSI is safe since message is solid
                    if (lsmi - *metadata.otrsi().unwrap().0) > below_max_depth {
                        should_promote = Some(false);
                        should_reattach = Some(true);
                    } else if (lsmi - *metadata.ytrsi().unwrap().0) > ytrsi_delta || (lsmi - otrsi_delta) > otrsi_delta
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
                parent_1_message_id: message.parent1().to_string(),
                parent_2_message_id: message.parent2().to_string(),
                is_solid,
                referenced_by_milestone_index,
                milestone_index,
                ledger_inclusion_state,
                conflict_reason,
                should_promote,
                should_reattach,
            })))
        }
        None => Err(reject::custom(NotFound("can not find message".to_string()))),
    }
}

/// Response of GET /api/v1/messages/{message_id}/metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageMetadataResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "parent1MessageId")]
    pub parent_1_message_id: String,
    #[serde(rename = "parent2MessageId")]
    pub parent_2_message_id: String,
    #[serde(rename = "isSolid")]
    pub is_solid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "referencedByMilestoneIndex")]
    pub referenced_by_milestone_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "milestoneIndex")]
    pub milestone_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ledgerInclusionState")]
    pub ledger_inclusion_state: Option<LedgerInclusionStateDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "conflictReason")]
    pub conflict_reason: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "shouldPromote")]
    pub should_promote: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "shouldReattach")]
    pub should_reattach: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LedgerInclusionStateDto {
    #[serde(rename = "conflicting")]
    Conflicting,
    #[serde(rename = "included")]
    Included,
    #[serde(rename = "noTransaction")]
    NoTransaction,
}

impl BodyInner for MessageMetadataResponse {}
