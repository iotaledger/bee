// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::BlockId;
use bee_ledger::workers::event::MilestoneConfirmed;
use serde::Serialize;

use crate::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConfirmedInfoResponse {
    id: String,
    excluded_ids: Vec<String>,
}

impl From<MilestoneConfirmed> for WsEvent {
    fn from(val: MilestoneConfirmed) -> Self {
        Self::new(WsTopic::ConfirmedInfo, WsEventInner::ConfirmedInfo(val.into()))
    }
}

impl From<MilestoneConfirmed> for ConfirmedInfoResponse {
    fn from(val: MilestoneConfirmed) -> Self {
        Self {
            id: val.message_id.to_string(),
            excluded_ids: val
                .excluded_no_transaction_messages
                .into_iter()
                .chain(
                    val.excluded_conflicting_messages
                        .into_iter()
                        .map(|v| v.0)
                        .collect::<Vec<BlockId>>(),
                )
                .collect::<Vec<BlockId>>()
                .iter()
                .map(|id| id.to_string())
                .collect(),
        }
    }
}
