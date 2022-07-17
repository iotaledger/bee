// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! MPS plugin for the Bee node.

#![warn(missing_docs)]

use std::convert::Infallible;

use async_trait::async_trait;
use bee_plugin::Plugin;
use bee_protocol::event::MpsMetricsUpdated;
use bee_runtime::event::Bus;
use log::info;

/// MPS plugin, for logging MPS metrics.
pub struct MpsPlugin;

#[async_trait]
impl Plugin for MpsPlugin {
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
