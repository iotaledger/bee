// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugin::{streamer::PluginStreamer, EventId, PluginId, UniqueId};

use bee_event_bus::EventBus;

use tokio::sync::oneshot::Sender;

use std::{collections::HashMap, process::Child};

macro_rules! spawn_streamers {
    ($plugin_id:ident, $event_id:ident, $bus:ident, $shutdown:ident, $($event_var:pat => $event_ty:ty),*) => {{
        match $event_id {
            $(
                $event_var => {
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<$event_ty>();

                tokio::spawn(async {
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

pub(crate) struct PluginHandler {
    shutdowns: HashMap<EventId, Sender<()>>,
    process: Child,
}

impl PluginHandler {
    pub(crate) fn new(process: Child) -> Self {
        Self {
            shutdowns: Default::default(),
            process,
        }
    }

    pub(crate) fn register_callback(
        &mut self,
        plugin_id: PluginId,
        event_id: EventId,
        bus: &EventBus<'static, UniqueId>,
    ) {
        if !self.shutdowns.contains_key(&event_id) {
            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
            self.shutdowns.insert(event_id, shutdown_tx);

            spawn_streamers!(plugin_id, event_id, bus, shutdown_rx, EventId::Dummy => ())
        }
    }

    pub(crate) fn shutdown(mut self, plugin_id: PluginId, bus: &EventBus<'static, UniqueId>) {
        for (_id, shutdown) in self.shutdowns {
            shutdown.send(()).unwrap();
        }
        bus.remove_listeners_with_id(plugin_id.into());

        // FIXME: send the shutdown signal via gRPC
        self.process.kill().unwrap();
    }
}
