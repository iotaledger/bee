// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugin::{handler::PluginHandler, EventId, PluginId, UniqueId};

use bee_event_bus::EventBus;

use std::{collections::HashMap, error::Error, process::Command, sync::Arc};

/// The bee node plugin manager.
pub(crate) struct PluginManager {
    /// Counter to create new and unique `PluginId` values.
    count: usize,
    /// Handlers for each plugin.
    handlers: HashMap<PluginId, PluginHandler>,
    /// Reference to the event bus.
    bus: Arc<EventBus<'static, UniqueId>>,
}

impl PluginManager {
    /// Creates a new and empty plugin manager.
    pub(crate) fn new(bus: Arc<EventBus<'static, UniqueId>>) -> Self {
        Self {
            count: 0,
            handlers: Default::default(),
            bus,
        }
    }

    /// Loads a new plugin from the specified command and returns the `PluginId` for that plugin.
    pub(crate) fn load_plugin(&mut self, mut command: Command) -> Result<PluginId, Box<dyn Error>> {
        let process = command.spawn()?;

        // FIXME: do the handshake and retrieve the event IDs.
        let event_ids: Vec<EventId> = vec![];

        let id = PluginId(self.count);
        let mut handler = PluginHandler::new(id, process);

        self.count += 1;

        for event_id in event_ids {
            handler.register_callback(event_id, &self.bus);
        }

        self.handlers.insert(id, handler);

        Ok(id)
    }

    /// UNloads a plugin with the specified identifier.
    pub(crate) fn unload_plugin(&mut self, id: PluginId) {
        if let Some(handler) = self.handlers.remove(&id) {
            handler.shutdown(&self.bus);
        }
    }
}
