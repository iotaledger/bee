// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{handler::PluginHandler, PluginError, PluginId, UniqueId};

use bee_event_bus::EventBus;

use log::{info, warn};
use tokio::process::Command;

use std::{collections::HashMap, sync::Arc};

/// The bee node plugin manager.
pub struct PluginManager {
    /// Counter to create new and unique [`PluginId`] values.
    counter: usize,
    /// Handlers for each plugin.
    handlers: HashMap<PluginId, PluginHandler>,
    /// Reference to the [`EventBus`].
    bus: Arc<EventBus<'static, UniqueId>>,
}

impl PluginManager {
    /// Creates a new and empty [`PluginManager`].
    pub fn new(bus: Arc<EventBus<'static, UniqueId>>) -> Self {
        Self {
            counter: 0,
            handlers: Default::default(),
            bus,
        }
    }

    pub(crate) fn generate_plugin_id(&mut self) -> PluginId {
        let plugin_id = PluginId(self.counter);

        self.counter += 1;

        plugin_id
    }

    /// Loads a new plugin from the specified [`Command`] and returns the [`PluginId`] for that plugin.
    pub async fn load(&mut self, command: Command) -> Result<PluginId, PluginError> {
        let plugin_id = self.generate_plugin_id();

        info!("loading plugin with identifier {}", plugin_id);
        let handler = PluginHandler::new(plugin_id, command, &self.bus).await?;
        info!("loaded plugin {}", handler.name());

        self.handlers.insert(plugin_id, handler);

        Ok(plugin_id)
    }

    /// Unloads a plugin with the specified [`PluginId`].
    pub async fn unload(&mut self, id: PluginId) -> Result<(), PluginError> {
        if let Some(handler) = self.handlers.remove(&id) {
            let name = handler.name().to_owned();

            info!("unloading plugin {}", name);
            handler.shutdown(&self.bus).await?;
            info!("plugin {} was successfully unloaded", name);
        } else {
            warn!("tried to unload an unknown plugin with identifier {}", id);
        }

        Ok(())
    }
}
