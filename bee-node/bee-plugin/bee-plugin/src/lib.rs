// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types for use in creating plugins for the Bee node.

#![warn(missing_docs)]

use std::{any::type_name, error::Error, fmt};

use async_trait::async_trait;
use bee_runtime::{event::Bus, node::Node, worker::Worker};

/// Plugin trait for necessary plugin operation.
#[async_trait]
pub trait Plugin: Sized + Send + Sync + 'static {
    /// Type used for plugin configuration.
    type Config: Send;
    /// Type of error that could be encountered by the plugin.
    type Error: Error;

    /// Starts the plugin using a given configuration.
    async fn start(config: Self::Config, bus: &Bus<'_>) -> Result<Self, Self::Error>;

    /// Stops the plugin.
    async fn stop(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// The node worker for the plugin.
pub struct PluginWorker<P: Plugin> {
    plugin: P,
}

/// Error during plugin operation.
pub struct PluginError<P: Plugin>(P::Error);

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
