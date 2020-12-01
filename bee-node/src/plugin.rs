// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::{event::Bus, node::Node, worker::Worker};

use async_trait::async_trait;

use std::{any::type_name, error::Error, fmt};

#[async_trait]
pub trait Plugin: Sized + Send + Sync + 'static {
    type Config: Send;
    type Error: Error;

    async fn start(config: Self::Config, bus: &Bus<'_>) -> Result<Self, Self::Error>;
    async fn stop(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct PluginWorker<P: Plugin> {
    plugin: P,
}

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
        let bus = node.resource();
        Ok(Self {
            plugin: P::start(config, &bus).await.map_err(PluginError)?,
        })
    }

    async fn stop(self, _node: &mut N) -> Result<(), Self::Error> {
        self.plugin.stop().await.map_err(PluginError)?;
        Ok(())
    }
}
