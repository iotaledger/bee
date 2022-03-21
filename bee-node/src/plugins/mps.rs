// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::convert::Infallible;

use async_trait::async_trait;
use bee_protocol::workers::event::MpsMetricsUpdated;
use bee_runtime::event::Bus;
use log::info;

use crate::plugins::Plugin;

pub struct Mps;

#[async_trait]
impl Plugin for Mps {
    type Config = ();
    type Error = Infallible;

    async fn start(_: Self::Config, bus: &Bus<'_>) -> Result<Self, Self::Error> {
        bus.add_listener::<(), MpsMetricsUpdated, _>(|metrics| {
            info!(
                "Mps: incoming {} new {} known {} invalid {} outgoing {}",
                metrics.incoming, metrics.new, metrics.known, metrics.invalid, metrics.outgoing
            );
        });
        Ok(Self)
    }
}
