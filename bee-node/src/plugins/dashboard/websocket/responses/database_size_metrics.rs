// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::{
    websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
    },
    workers::db_size_metrics::DatabaseSizeMetrics,
};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DatabaseSizeMetricsResponse {
    pub total: u64,
    pub ts: u64,
}

impl From<DatabaseSizeMetrics> for WsEvent {
    fn from(val: DatabaseSizeMetrics) -> Self {
        Self::new(
            WsTopic::DatabaseSizeMetrics,
            WsEventInner::DatabaseSizeMetrics(val.into()),
        )
    }
}

impl From<DatabaseSizeMetrics> for DatabaseSizeMetricsResponse {
    fn from(val: DatabaseSizeMetrics) -> Self {
        Self {
            total: val.total,
            ts: val.ts,
        }
    }
}
