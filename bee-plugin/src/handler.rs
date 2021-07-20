// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{streamer::PluginStreamer, EventId, PluginId, UniqueId};

use bee_event_bus::EventBus;

use tokio::{
    spawn,
    sync::{mpsc::unbounded_channel, oneshot::Sender},
};

use std::{
    collections::{hash_map::Entry, HashMap},
    process::Child,
};

macro_rules! spawn_streamers {
    ($plugin_id:expr, $event_id:ident, $bus:ident, $shutdown:ident, $($event_var:pat => $event_ty:ty),*) => {{
        match $event_id {
            $(
                $event_var => {
                    let (tx, rx) = unbounded_channel::<$event_ty>();

                    spawn(async {
                        let mut streamer = PluginStreamer::new(rx, $shutdown);
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
    /// The identifier of the plugin.
    id: PluginId,
}

impl PluginHandler {
    /// Creates a new plugin handler from a process running the plugin logic.
    pub(crate) fn new(id: PluginId, process: Child) -> Self {
        Self {
            shutdowns: Default::default(),
            process,
            id,
        }
    }

    /// Registers a callback for an event with the specified `EventId` in the event bus.
    pub(crate) fn register_callback(&mut self, event_id: EventId, bus: &EventBus<'static, UniqueId>) {
        if let Entry::Vacant(entry) = self.shutdowns.entry(event_id) {
            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
            entry.insert(shutdown_tx);

            spawn_streamers!(self.id, event_id, bus, shutdown_rx, EventId::Dummy => ())
        }
    }

    /// Shutdowns the plugin by shutting down all the plugin streamers, removing the plugin
    /// callbacks from the event bus and killing the plugin process.
    pub(crate) fn shutdown(mut self, bus: &EventBus<'static, UniqueId>) {
        for (_id, shutdown) in self.shutdowns {
            shutdown.send(()).unwrap();
        }
        bus.remove_listeners_with_id(self.id.into());

        // FIXME: send the shutdown signal via gRPC
        self.process.kill().unwrap();
    }
}
