// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{
        confirmed_info::ConfirmedInfoResponse, confirmed_milestone_metrics::ConfirmedMilestoneMetricsResponse,
        database_size_metrics::DatabaseSizeMetricsResponse, milestone::MilestoneResponse,
        milestone_info::MilestoneInfoResponse, mps_metrics_updated::MpsMetricsUpdatedResponse,
        node_status::NodeStatusResponse, solid_info::SolidInfoResponse,
        sync_status::SyncStatusResponse, tip_info::TipInfoResponse, vertex::VertexResponse,
    },
    topics::WsTopic,
};

use bee_rest_api::handlers::peers::PeersResponse;

use serde::Serialize;

pub(crate) mod confirmed_info;
pub(crate) mod confirmed_milestone_metrics;
pub(crate) mod database_size_metrics;
pub(crate) mod milestone;
pub(crate) mod milestone_info;
pub(crate) mod mps_metrics_updated;
pub(crate) mod node_status;
pub(crate) mod peer_metric;
pub(crate) mod solid_info;
pub(crate) mod sync_status;
pub(crate) mod tip_info;
pub(crate) mod vertex;

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
    ConfirmedInfo(ConfirmedInfoResponse),
    ConfirmedMilestoneMetrics(ConfirmedMilestoneMetricsResponse),
    MilestoneInfo(MilestoneInfoResponse),
    Vertex(VertexResponse),
    DatabaseSizeMetrics(DatabaseSizeMetricsResponse),
    TipInfo(TipInfoResponse),
    NodeStatus(NodeStatusResponse),
    PeerMetric(PeersResponse),
}
