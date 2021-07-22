// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::PluginError,
    grpc::{plugin_client::PluginClient, EventId, HandshakeRequest},
    handler::PluginHandler,
    PluginId, UniqueId,
};

use bee_event_bus::EventBus;

use tokio::{
    process::Command,
    time::{sleep, Duration},
};
use tonic::Request;

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
    pub async fn load_plugin(&mut self, mut command: Command) -> Result<PluginId, PluginError> {
        command.kill_on_drop(true);

        let id = PluginId(self.count);

        log::info!("spawning command `{:?}` for the plugin `{:?}`", command, id);
        let process = command.spawn()?;

        sleep(Duration::from_secs(1)).await;

        log::info!("connecting to the plugin `{:?}`", id);
        let mut client = PluginClient::connect("http://[::1]:50051").await?;

        log::info!("handshaking with the plugin `{:?}`", id);
        let handshake_response = client.handshake(Request::new(HandshakeRequest {})).await?;

        let raw_ids: Vec<i32> = handshake_response.into_inner().ids;

        let mut handler = PluginHandler::new(id, process, client);

        for raw_id in raw_ids {
            match EventId::from_i32(raw_id) {
                Some(id) => handler.register_callback(id, &self.bus),
                None => return Err(PluginError::InvalidEventId(raw_id)),
            };
        }

        self.handlers.insert(id, handler);

        self.count += 1;

        Ok(id)
    }

    /// Unloads a plugin with the specified identifier.
    pub async fn unload_plugin(&mut self, id: PluginId) -> Result<(), PluginError> {
        if let Some(handler) = self.handlers.remove(&id) {
            log::info!("shutting down the plugin `{:?}`", id);
            handler.shutdown(&self.bus).await?;
            log::info!("the plugin `{:?}` was shut down successfully", id);
        }

        Ok(())
    }
}
