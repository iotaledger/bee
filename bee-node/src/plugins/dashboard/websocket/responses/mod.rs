// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{
        milestone::MilestoneResponse, milestone_info::MilestoneInfoResponse,
        mps_metrics_updated::MpsMetricsUpdatedResponse, solid_info::SolidInfoResponse, sync_status::SyncStatusResponse,
    },
    topics::WsTopic,
};

use serde::Serialize;

pub(crate) mod milestone;
pub(crate) mod milestone_info;
pub(crate) mod mps_metrics_updated;
pub(crate) mod solid_info;
pub(crate) mod sync_status;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct WsEvent {
    #[serde(rename = "type")]
    pub(crate) kind: WsTopic,
    #[serde(rename = "data")]
    pub(crate) inner: WsEventInner,
}

impl WsEvent {
    pub(crate) fn new(kind: WsTopic, inner: WsEventInner) -> Self {
        Self { kind, inner }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum WsEventInner {
    SyncStatus(SyncStatusResponse),
    MpsMetricsUpdated(MpsMetricsUpdatedResponse),
    Milestone(MilestoneResponse),
    SolidInfo(SolidInfoResponse),
    MilestoneInfo(MilestoneInfoResponse),
}
