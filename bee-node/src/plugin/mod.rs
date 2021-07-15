// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_event_bus::EventBus;

use tokio::sync::{
    mpsc::UnboundedReceiver,
    oneshot::{Receiver, Sender},
};

use std::{
    any::TypeId,
    collections::HashMap,
    error::Error,
    process::{Child, Command},
    sync::Arc,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum UniqueId {
    Type(TypeId),
    Plugin(PluginId),
}

impl From<TypeId> for UniqueId {
    fn from(id: TypeId) -> Self {
        Self::Type(id)
    }
}

struct PluginStreamer<T> {
    rx: UnboundedReceiver<T>,
    shutdown: Receiver<()>,
}

impl<T> PluginStreamer<T> {
    fn new(rx: UnboundedReceiver<T>, shutdown: Receiver<()>) -> Self {
        Self { rx, shutdown }
    }

    async fn run(&mut self) {
        loop {
            tokio::select! {
                _ = &mut self.shutdown => break,
                event = self.rx.recv() => match event {
                    Some(_event) => println!("lol event"),
                    None => break,
                }
            }
        }
    }
}

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

struct PluginHandler {
    shutdowns: HashMap<EventId, Sender<()>>,
    process: Child,
}

impl PluginHandler {
    fn new(process: Child) -> Self {
        Self {
            shutdowns: Default::default(),
            process,
        }
    }

    fn register_callback(&mut self, plugin_id: PluginId, event_id: EventId, bus: &EventBus<'static, UniqueId>) {
        if !self.shutdowns.contains_key(&event_id) {
            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
            self.shutdowns.insert(event_id, shutdown_tx);

            spawn_streamers!(plugin_id, event_id, bus, shutdown_rx, EventId::Dummy => ())
        }
    }

    fn shutdown(mut self, plugin_id: PluginId, bus: &EventBus<'static, UniqueId>) {
        for (_id, shutdown) in self.shutdowns {
            shutdown.send(()).unwrap();
        }
        bus.remove_listeners_with_id(plugin_id);

        // FIXME: send the shutdown signal via gRPC
        self.process.kill().unwrap();
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct PluginId(usize);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum EventId {
    Dummy,
}

struct PluginManager {
    count: usize,
    handlers: HashMap<PluginId, PluginHandler>,
    bus: Arc<EventBus<'static, UniqueId>>,
}

impl PluginManager {
    fn new(bus: Arc<EventBus<'static, UniqueId>>) -> Self {
        Self {
            count: 0,
            handlers: Default::default(),
            bus,
        }
    }

    fn load_plugin(&mut self, mut command: Command) -> Result<PluginId, Box<dyn Error>> {
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

    fn unload_plugin(&mut self, id: PluginId) {
        if let Some(handler) = self.handlers.remove(&id) {
            handler.shutdown(id, &self.bus);
        }
    }
}
