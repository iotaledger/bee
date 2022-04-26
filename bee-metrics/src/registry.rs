// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use parking_lot::RwLock;
use prometheus_client::registry::Registry as PrometheusRegistry;

use crate::encoding::SendSyncEncodeMetric;

/// A type used to register metrics so they can be scraped later.
pub struct Registry {
    pub(crate) registry: RwLock<PrometheusRegistry<Box<dyn SendSyncEncodeMetric + 'static>>>,
}

impl Registry {
    /// Registers a new metric with a name and a help message.
    pub fn register(
        &self,
        name: impl Into<String>,
        help: impl Into<String>,
        metric: impl SendSyncEncodeMetric + 'static,
    ) {
        self.registry.write().register(name, help, Box::new(metric))
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            registry: RwLock::new(Default::default()),
        }
    }
}