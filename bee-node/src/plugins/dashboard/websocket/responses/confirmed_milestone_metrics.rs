// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::{
    websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
    },
    workers::confirmed_ms_metrics::ConfirmedMilestoneMetrics,
};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConfirmedMilestoneMetricsResponse {
    ms_index: u32,
    mps: u64,
    rmps: u64,
    referenced_rate: f64,
    time_since_last_ms: u64,
}

pub(crate) fn forward(message: ConfirmedMilestoneMetrics) -> WsEvent {
    message.into()
}

impl From<ConfirmedMilestoneMetrics> for WsEvent {
    fn from(val: ConfirmedMilestoneMetrics) -> Self {
        Self::new(
            WsTopic::ConfirmedMilestoneMetrics,
            WsEventInner::ConfirmedMilestoneMetrics(val.into()),
        )
    }
}

impl From<ConfirmedMilestoneMetrics> for ConfirmedMilestoneMetricsResponse {
    fn from(val: ConfirmedMilestoneMetrics) -> Self {
        Self {
            ms_index: val.ms_index,
            mps: val.mps,
            rmps: val.rmps,
            referenced_rate: val.referenced_rate,
            time_since_last_ms: val.time_since_last_ms,
        }
    }
}
