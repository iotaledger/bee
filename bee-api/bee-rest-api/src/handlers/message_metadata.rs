// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::{NotFound, ServiceUnavailable},
    handlers::{EnvelopeContent, SuccessEnvelope},
    storage::Backend,
};

use bee_common_pt2::node::ResHandle;
use bee_message::prelude::*;
use bee_protocol::tangle::MsTangle;

use serde::Serialize;
use warp::{reject, Rejection, Reply};

pub(crate) async fn message_metadata<B: Backend>(
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

                    Ok(warp::reply::json(&SuccessEnvelope::new(res)))
                }
                None => Err(reject::custom(NotFound("can not find data".to_string()))),
            }
        }
        None => Err(reject::custom(NotFound("can not find data".to_string()))),
    }
}

/// Response of GET /api/v1/messages/{message_id}/metadata
#[derive(Clone, Debug, Serialize)]
pub struct GetMessageMetadataResponse {
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
    #[serde(rename = "ledgerInclusionState")]
    pub ledger_inclusion_state: Option<LedgerInclusionStateDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "shouldPromote")]
    pub should_promote: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "shouldReattach")]
    pub should_reattach: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum LedgerInclusionStateDto {
    Conflicting,
    Included,
    NoTransaction,
}

impl EnvelopeContent for GetMessageMetadataResponse {}
