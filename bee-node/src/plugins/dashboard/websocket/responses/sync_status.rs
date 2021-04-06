// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    plugins::dashboard::websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
    },
    storage::StorageBackend,
};

use bee_ledger::consensus::event::MilestoneConfirmed;
use bee_tangle::{event::LatestMilestoneChanged, MsTangle};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SyncStatusResponse {
    pub(crate) lmi: u32,
    pub(crate) cmi: u32,
}

pub(crate) fn forward_latest_milestone_changed<B: StorageBackend>(
    latest_milestone: LatestMilestoneChanged,
    tangle: &MsTangle<B>,
) -> WsEvent {
    WsEvent::new(
        WsTopic::SyncStatus,
        WsEventInner::SyncStatus(SyncStatusResponse {
            lmi: *latest_milestone.index,
            cmi: *tangle.get_confirmed_milestone_index(),
        }),
    )
}

pub(crate) fn forward_confirmed_milestone_changed<B: StorageBackend>(
    event: &MilestoneConfirmed,
    tangle: &MsTangle<B>,
) -> WsEvent {
    WsEvent::new(
        WsTopic::SyncStatus,
        WsEventInner::SyncStatus(SyncStatusResponse {
            lmi: *tangle.get_latest_milestone_index(),
            cmi: *event.index,
        }),
    )
}
