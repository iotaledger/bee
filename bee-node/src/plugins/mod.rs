// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "dashboard")]
#[allow(missing_docs)]
pub mod dashboard;
#[allow(missing_docs)]
pub mod mps;
#[allow(missing_docs)]
pub mod mqtt;

#[cfg(feature = "dashboard")]
pub use dashboard::Dashboard;
pub use mps::Mps;
pub use mqtt::Mqtt;

use bee_runtime::{event::Bus, node::Node, worker::Worker};

use async_trait::async_trait;

use std::{any::type_name, error::Error, fmt};

/// A trait to be implemented by node plugins.
#[async_trait]
pub trait Plugin: Sized + Send + Sync + 'static {
    /// The type containing the configuration state for this plugin.
    type Config: Send;

    /// The error type that may be emitted during plugin start or stop.
    type Error: Error;

    /// The function to be invoked on plugin start. The plugin should start all worker tasks and bind all event
    /// handlers in this function.
    async fn start(config: Self::Config, bus: &Bus<'_>) -> Result<Self, Self::Error>;

    /// The method to be invoked on plugin shutdown. This function should shut down all worker tasks and unbind event
    /// handlers.
    async fn stop(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub(crate) struct PluginWorker<P: Plugin> {
    plugin: P,
}

pub(crate) struct PluginError<P: Plugin>(P::Error);

impl<P: Plugin> fmt::Debug for PluginError<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Plugin `{}` error: {:?}", type_name::<P>(), self.0)
    }
}

impl<P: Plugin> fmt::Display for PluginError<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Plugin `{}` error: {}", type_name::<P>(), self.0)
    }
}

impl<P: Plugin> Error for PluginError<P> {}

#[async_trait]
impl<P: Plugin, N: Node> Worker<N> for PluginWorker<P> {
    type Config = P::Config;
    type Error = PluginError<P>;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let bus = node.bus();
        Ok(Self {
            plugin: P::start(config, &bus).await.map_err(PluginError)?,
        })
    }

    async fn stop(self, _node: &mut N) -> Result<(), Self::Error> {
        self.plugin.stop().await.map_err(PluginError)?;
        Ok(())
    }
}
