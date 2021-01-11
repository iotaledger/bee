// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

use bee_ledger::event::MilestoneConfirmed;
use bee_message::MessageId;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConfirmedInfoResponse {
    id: String,
    excluded_ids: Vec<String>,
}

pub(crate) fn forward(message: MilestoneConfirmed) -> WsEvent {
    message.into()
}

impl From<MilestoneConfirmed> for WsEvent {
    fn from(val: MilestoneConfirmed) -> Self {
        Self::new(WsTopic::ConfirmedInfo, WsEventInner::ConfirmedInfo(val.into()))
    }
}

impl From<MilestoneConfirmed> for ConfirmedInfoResponse {
    fn from(val: MilestoneConfirmed) -> Self {
        Self {
            id: val.id.to_string(),
            excluded_ids: val
                .excluded_no_transaction_messages
                .into_iter()
                .chain(val.excluded_conflicting_messages.into_iter())
                .collect::<Vec<MessageId>>()
                .iter()
                .map(|id| id.to_string())
                .collect(),
        }
    }
}
