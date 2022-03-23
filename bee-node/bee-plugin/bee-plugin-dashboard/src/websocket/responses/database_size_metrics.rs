// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Serialize;

use crate::{
    websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
    },
    workers::db_size_metrics::DatabaseSizeMetrics,
};

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
