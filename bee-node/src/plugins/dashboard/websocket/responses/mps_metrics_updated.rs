// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::{
    broadcast,
    websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
        WsUsers,
    },
    Dashboard,
};

use bee_common::event::Bus;
use bee_common_pt2::node::ResHandle;
use bee_protocol::event::MpsMetricsUpdated;

use futures::executor::block_on;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct MpsMetricsUpdatedResponse(pub MpsMetricsUpdatedDto);

#[derive(Clone, Debug, Serialize)]
pub(crate) struct MpsMetricsUpdatedDto {
    pub incoming: u64,
    pub new: u64,
    pub known: u64,
    pub invalid: u64,
    pub outgoing: u64,
}

impl From<&MpsMetricsUpdated> for WsEvent {
    fn from(val: &MpsMetricsUpdated) -> Self {
        Self::new(WsTopic::MPSMetrics, WsEventInner::MpsMetricsUpdated(val.into()))
    }
}

impl From<&MpsMetricsUpdated> for MpsMetricsUpdatedResponse {
    fn from(val: &MpsMetricsUpdated) -> Self {
        Self(val.into())
    }
}

impl From<&MpsMetricsUpdated> for MpsMetricsUpdatedDto {
    fn from(val: &MpsMetricsUpdated) -> Self {
        MpsMetricsUpdatedDto {
            incoming: val.incoming,
            new: val.new,
            known: val.known,
            invalid: val.invalid,
            outgoing: val.outgoing,
        }
    }
}

pub(crate) fn register(bus: ResHandle<Bus>, users: WsUsers) {
    bus.add_listener::<Dashboard, MpsMetricsUpdated, _>(move |metrics| {
        block_on(broadcast(metrics.into(), users.clone()))
    });
}
