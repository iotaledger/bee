// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_plugin::Plugin;

use bee_protocol::workers::event::MpsMetricsUpdated;
use bee_runtime::event::Bus;

use async_trait::async_trait;
use log::info;

use std::convert::Infallible;

pub struct MpsPlugin;

#[async_trait]
impl Plugin for MpsPlugin {
    type Config = ();
    type Error = Infallible;

    async fn start(_: Self::Config, bus: &Bus<'_>) -> Result<Self, Self::Error> {
        bus.add_listener::<(), MpsMetricsUpdated, _>(|metrics| {
            info!(
                "incoming {} new {} known {} invalid {} outgoing {}",
                metrics.incoming, metrics.new, metrics.known, metrics.invalid, metrics.outgoing
            );
        });
        Ok(Self)
    }
}
