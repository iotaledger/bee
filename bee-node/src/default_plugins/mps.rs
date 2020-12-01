// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugin::Plugin;

use bee_common::event::Bus;
use bee_protocol::event::TpsMetricsUpdated;

use async_trait::async_trait;
use log::info;

use std::convert::Infallible;

pub struct Mps;

#[async_trait]
impl Plugin for Mps {
    type Config = ();
    type Error = Infallible;

    async fn start(_: Self::Config, bus: &Bus<'_>) -> Result<Self, Self::Error> {
        bus.add_listener::<(), TpsMetricsUpdated, _>(|metrics| {
            info!(
                "incoming {} new {} known {} invalid {} outgoing {}",
                metrics.incoming, metrics.new, metrics.known, metrics.invalid, metrics.outgoing
            );
        });
        Ok(Self)
    }
}
