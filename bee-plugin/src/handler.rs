// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    grpc::{plugin_client::PluginClient, DummyEvent, EventId, ShutdownRequest},
    streamer::PluginStreamer,
    PluginId, UniqueId,
};

use bee_event_bus::EventBus;

use tokio::{
    process::Child,
    select, spawn,
    sync::{mpsc::unbounded_channel, oneshot::Sender},
    time::sleep,
};
use tonic::{transport::Channel, Request};

use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    time::Duration,
};

macro_rules! spawn_streamers {
    ($plugin_id:expr, $event_id:ident, $bus:ident, $client:expr, $shutdown:ident, $($event_var:pat => $event_ty:ty),*) => {{
        match $event_id {
            $(
                $event_var => {
                    let (tx, rx) = unbounded_channel::<$event_ty>();

                    let client = $client.clone();

                    spawn(async move {
                        let mut streamer = PluginStreamer::new(rx, $shutdown, client);
                        streamer.run().await;
                    });

                    $bus.add_listener_with_id(move |event: &$event_ty| {
                        tx.send(event.clone()).unwrap()
                    }, UniqueId::Plugin($plugin_id));
                }
            )*
        }
    }};
}

/// A handler for a plugin.
pub(crate) struct PluginHandler {
    /// Shutdown for every `PluginStreamer` used by the plugin.
    shutdowns: HashMap<EventId, Sender<()>>,
    /// The OS process running the plugin.
    process: Child,
    client: PluginClient<Channel>,
    /// The identifier of the plugin.
    id: PluginId,
}

impl PluginHandler {
    /// Creates a new plugin handler from a process running the plugin logic.
    pub(crate) fn new(id: PluginId, process: Child, client: PluginClient<Channel>) -> Self {
        Self {
            shutdowns: Default::default(),
            process,
            client,
            id,
        }
    }

    /// Registers a callback for an event with the specified `EventId` in the event bus.
    pub(crate) fn register_callback(&mut self, event_id: EventId, bus: &EventBus<'static, UniqueId>) {
        if let Entry::Vacant(entry) = self.shutdowns.entry(event_id) {
            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
            entry.insert(shutdown_tx);

            spawn_streamers!(self.id, event_id, bus, &self.client, shutdown_rx, EventId::Dummy => DummyEvent)
        }
    }

    /// Shutdowns the plugin by shutting down all the plugin streamers, removing the plugin
    /// callbacks from the event bus and killing the plugin process.
    pub(crate) async fn shutdown(mut self, bus: &EventBus<'static, UniqueId>) -> Result<(), Box<dyn Error>> {
        for (_id, shutdown) in self.shutdowns {
            shutdown.send(()).unwrap();
        }
        bus.remove_listeners_with_id(self.id.into());

        let shutdown = self.client.shutdown(Request::new(ShutdownRequest {}));
        let delay = sleep(Duration::from_secs(30));

        select! {
            result = shutdown => {
                result?;
            },
            _ = delay => (),
        }

        self.process.kill().await?;

        Ok(())
    }
}
