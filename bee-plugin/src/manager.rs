// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::PluginError, handler::PluginHandler, PluginId, UniqueId};

use bee_event_bus::EventBus;

use tokio::process::Command;

use std::{collections::HashMap, sync::Arc};

/// The bee node plugin manager.
pub struct PluginManager {
    /// Counter to create new and unique `PluginId` values.
    count: usize,
    /// Handlers for each plugin.
    handlers: HashMap<PluginId, PluginHandler>,
    /// Reference to the event bus.
    bus: Arc<EventBus<'static, UniqueId>>,
}

impl PluginManager {
    /// Creates a new and empty plugin manager.
    pub fn new(bus: Arc<EventBus<'static, UniqueId>>) -> Self {
        Self {
            count: 0,
            handlers: Default::default(),
            bus,
        }
    }

    /// Loads a new plugin from the specified command and returns the `PluginId` for that plugin.
    pub async fn load_plugin(&mut self, command: Command) -> Result<PluginId, PluginError> {
        let plugin_id = PluginId(self.count);

        let handler = PluginHandler::new(plugin_id, command, &self.bus).await?;

        self.handlers.insert(plugin_id, handler);

        self.count += 1;

        Ok(plugin_id)
    }

    /// Unloads a plugin with the specified identifier.
    pub async fn unload_plugin(&mut self, id: PluginId) -> Result<(), PluginError> {
        if let Some(handler) = self.handlers.remove(&id) {
            let name = handler.name().to_owned();
            log::info!("shutting down the `{}` plugin", name);
            handler.shutdown(&self.bus).await?;
            log::info!("the `{}` plugin was shut down successfully", name);
        }

        Ok(())
    }
}
