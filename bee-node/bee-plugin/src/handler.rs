// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::PluginError,
    event::EventId,
    grpc::{plugin_client::PluginClient, DummyEvent, ShutdownRequest, SillyEvent},
    handshake::HandshakeInfo,
    streamer::PluginStreamer,
    PluginId, UniqueId,
};

use bee_event_bus::EventBus;

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    select, spawn,
    sync::{mpsc::unbounded_channel, oneshot::Sender},
    task::JoinHandle,
    time::sleep,
};
use tonic::{transport::Channel, Request};

use std::{
    any::type_name,
    collections::{hash_map::Entry, HashMap},
    process::Stdio,
    time::Duration,
};

macro_rules! spawn_streamers {
    ($self:ident, $event_id:ident, $bus:ident, $shutdown:ident, $($event_var:pat => $event_ty:ty),*) => {{
        match $event_id {
            $(
                $event_var => {
                    let (tx, rx) = unbounded_channel::<$event_ty>();

                    let client = $self.client.clone();

                    spawn(async move {
                        PluginStreamer::new(rx, $shutdown, client).run().await;
                    });

                    log::info!("registering `{}` callback for the `{}` plugin", type_name::<$event_ty>(), $self.name);
                    $bus.add_listener_with_id(move |event: &$event_ty| {
                        if let Err(err) = tx.send(event.clone()) {
                            log::warn!("failed to send event: {}", err);
                        }
                    }, UniqueId::Plugin($self.plugin_id));
                }
            )*
        }
    }};
}

/// A handler for a plugin.
pub(crate) struct PluginHandler {
    /// The identifier of the plugin.
    plugin_id: PluginId,
    /// The name of the plugin.
    name: String,
    /// Shutdown for every `PluginStreamer` used by the plugin.
    shutdowns: HashMap<EventId, Sender<()>>,
    /// The OS process running the plugin.
    process: Child,
    /// The gRPC client.
    client: PluginClient<Channel>,
    /// The task handling stdio redirection.
    stdio_task: JoinHandle<Result<(), std::io::Error>>,
}

impl PluginHandler {
    /// Creates a new plugin handler from a process running the plugin logic.
    pub(crate) async fn new(
        plugin_id: PluginId,
        mut command: Command,
        bus: &EventBus<'static, UniqueId>,
    ) -> Result<Self, PluginError> {
        command.kill_on_drop(true).stdout(Stdio::piped()).stderr(Stdio::piped());

        log::info!(
            "spawning command `{:?}` for the plugin with ID `{:?}`",
            command,
            plugin_id
        );
        let mut process = command.spawn()?;

        // stderr and stdout are guaranteed to be `Some` because we piped them in the command.
        let stderr = BufReader::new(process.stderr.take().unwrap());
        let mut stdout = BufReader::new(process.stdout.take().unwrap());

        let mut buf = String::new();
        stdout.read_line(&mut buf).await?;
        let handshake_info = HandshakeInfo::parse(&buf)?;

        let name = format!("{}-{}", handshake_info.name, plugin_id.0);
        let target = format!("plugins::{}", name);
        let stdio_task = tokio::spawn(async move {
            let mut stdout_lines = stdout.lines();
            let mut stderr_lines = stderr.lines();

            loop {
                tokio::select! {
                    res = stdout_lines.next_line() => match res? {
                        Some(line) => {
                            log::info!(target: &target, "{}", line);
                        },
                        None => break,
                    },
                    res = stderr_lines.next_line() => match res? {
                        Some(line) => {
                            log::warn!(target: &target, "{}", line);
                        },
                        None => break,
                    }
                }
            }

            Ok(())
        });

        let address = format!("http://{}/", handshake_info.address);
        log::info!("connecting to the `{}` plugin at `{}`", name, address);
        let client = async {
            let mut count = 0;
            loop {
                match PluginClient::connect(address.clone()).await {
                    Ok(client) => break Ok(client),
                    Err(err) => {
                        log::warn!("connection to the `{}` plugin failed: {}", name, err);
                        if count == 5 {
                            log::warn!("connection to the `{}` plugin will not be retried anymore", name);
                            break Err(err);
                        } else {
                            let secs = 5u64.pow(count);
                            log::warn!(
                                "connection to the `{}` plugin will be retried in {} seconds",
                                name,
                                secs
                            );
                            tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
                            count += 1;
                        }
                    }
                }
            }
        }
        .await?;
        log::info!("connection to the `{}` plugin was successful", name);

        let mut this = Self {
            plugin_id,
            name,
            process,
            client,
            shutdowns: Default::default(),
            stdio_task,
        };

        for event_id in handshake_info.event_ids {
            this.register_callback(event_id, bus);
        }

        Ok(this)
    }

    /// Registers a callback for an event with the specified `EventId` in the event bus.
    fn register_callback(&mut self, event_id: EventId, bus: &EventBus<'static, UniqueId>) {
        if let Entry::Vacant(entry) = self.shutdowns.entry(event_id) {
            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
            entry.insert(shutdown_tx);

            spawn_streamers! {
                self, event_id, bus, shutdown_rx,
                EventId::Dummy => DummyEvent,
                EventId::Silly => SillyEvent
            }
        }
    }

    /// Shutdowns the plugin by shutting down all the plugin streamers, removing the plugin
    /// callbacks from the event bus and killing the plugin process.
    pub(crate) async fn shutdown(mut self, bus: &EventBus<'static, UniqueId>) -> Result<(), PluginError> {
        for (_id, shutdown) in self.shutdowns {
            // If sending fails, this means that the receiver was already dropped which means that
            // the streamer is already gone.
            shutdown.send(()).ok();
        }
        log::info!("removing callbacks for the `{}` plugin", self.name);
        bus.remove_listeners_with_id(self.plugin_id.into());

        log::info!("sending shutdown request to the `{}` plugin", self.name);
        let shutdown = self.client.shutdown(Request::new(ShutdownRequest {}));
        let delay = sleep(Duration::from_secs(30));

        select! {
            result = shutdown => {
                result?;
            },
            _ = delay => {
                log::warn!("the shutdown request for the `{}` plugin timed out", self.name);
            },
        }

        self.stdio_task.abort();
        if let Err(err) = self.stdio_task.await {
            if err.is_panic() {
                log::warn!("stdio redirection for the `{}` plugin panicked: {}", self.name, err);
            }
        };

        log::info!("killing process for the `{}` plugin", self.name);
        self.process.kill().await?;

        Ok(())
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}
