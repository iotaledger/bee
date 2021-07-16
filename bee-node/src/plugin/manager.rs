// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugin::{handler::PluginHandler, EventId, PluginId, UniqueId};

use bee_event_bus::EventBus;

use std::{collections::HashMap, error::Error, process::Command, sync::Arc};

pub(crate) struct PluginManager {
    count: usize,
    handlers: HashMap<PluginId, PluginHandler>,
    bus: Arc<EventBus<'static, UniqueId>>,
}

impl PluginManager {
    pub(crate) fn new(bus: Arc<EventBus<'static, UniqueId>>) -> Self {
        Self {
            count: 0,
            handlers: Default::default(),
            bus,
        }
    }

    pub(crate) fn load_plugin(&mut self, mut command: Command) -> Result<PluginId, Box<dyn Error>> {
        let process = command.spawn()?;

        // FIXME: do the handshake and retrieve the event IDs.
        let event_ids: Vec<EventId> = vec![];

        let mut handler = PluginHandler::new(process);

        self.count += 1;
        let id = PluginId(self.count);

        for event_id in event_ids {
            handler.register_callback(id, event_id, &self.bus);
        }

        self.handlers.insert(id, handler);

        Ok(id)
    }

    pub(crate) fn unload_plugin(&mut self, id: PluginId) {
        if let Some(handler) = self.handlers.remove(&id) {
            handler.shutdown(id, &self.bus);
        }
    }
}
