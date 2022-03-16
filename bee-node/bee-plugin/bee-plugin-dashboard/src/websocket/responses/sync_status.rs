// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    storage::StorageBackend,
    websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
    },
};

use bee_ledger::workers::event::MilestoneConfirmed;
use bee_tangle::{event::LatestMilestoneChanged, Tangle};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SyncStatusResponse {
    pub(crate) lmi: u32,
    pub(crate) cmi: u32,
}

pub(crate) fn forward_latest_milestone_changed<S: StorageBackend>(
    latest_milestone: LatestMilestoneChanged,
    tangle: &Tangle<S>,
) -> WsEvent {
    WsEvent::new(
        WsTopic::SyncStatus,
        WsEventInner::SyncStatus(SyncStatusResponse {
            lmi: *latest_milestone.index,
            cmi: *tangle.get_confirmed_milestone_index(),
        }),
    )
}

pub(crate) fn forward_confirmed_milestone_changed<S: StorageBackend>(
    event: &MilestoneConfirmed,
    tangle: &Tangle<S>,
) -> WsEvent {
    WsEvent::new(
        WsTopic::SyncStatus,
        WsEventInner::SyncStatus(SyncStatusResponse {
            lmi: *tangle.get_latest_milestone_index(),
            cmi: *event.index,
        }),
    )
}