// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    plugins::dashboard::websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
    },
    storage::StorageBackend,
};

use bee_protocol::event::LatestMilestoneChanged;
use bee_tangle::MsTangle;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SyncStatusResponse {
    lmi: u32,
    lsmi: u32, // Shouldn't it be `smi` (solid milestone index) instead?
}

pub(crate) fn forward_latest_milestone_changed<B: StorageBackend>(latest_milestone: LatestMilestoneChanged, tangle: &MsTangle<B>) -> WsEvent {
    WsEvent::new(
        WsTopic::SyncStatus,
        WsEventInner::SyncStatus(SyncStatusResponse {
            lmi: *latest_milestone.index,
            lsmi: *tangle.get_latest_solid_milestone_index(),
        }),
    )
}

pub(crate) fn forward_solid_milestone_changed<B: Backend>(
    latest_solid_milestone: LatestSolidMilestoneChanged,
    tangle: &MsTangle<B>,
) -> WsEvent {
    WsEvent::new(
        WsTopic::SyncStatus,
        WsEventInner::SyncStatus(SyncStatusResponse {
            lmi: *tangle.get_latest_milestone_index(),
            lsmi: *latest_solid_milestone.index,
        }),
    )
}
