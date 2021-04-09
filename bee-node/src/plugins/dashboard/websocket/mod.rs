// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod commands;
pub(crate) mod responses;
mod topics;

use commands::WsCommand;
use topics::WsTopic;

use crate::{
    plugins::dashboard::{
        auth::{jwt::JsonWebToken, AUDIENCE_CLAIM},
        config::DashboardAuthConfig,
        websocket::responses::{
            database_size_metrics::DatabaseSizeMetricsResponse, sync_status::SyncStatusResponse, WsEvent, WsEventInner,
        },
    },
    storage::StorageBackend,
};

use bee_runtime::{resource::ResourceHandle, shutdown_stream::ShutdownStream};
use bee_tangle::MsTangle;

use futures::{channel::oneshot, FutureExt, StreamExt};
use log::{debug, error};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

pub(crate) struct WsUser {
    pub(crate) tx: mpsc::UnboundedSender<Result<Message, warp::Error>>,
    pub(crate) shutdown: Option<oneshot::Sender<()>>,
    pub(crate) topics: HashSet<WsTopic>,
}

impl WsUser {
    pub(crate) fn send(&self, event: WsEvent) {
        match serde_json::to_string(&event) {
            Ok(as_text) => {
                if let Err(_) = self.tx.send(Ok(Message::text(as_text))) {
                    // The tx is disconnected, our `user_disconnected` code should be happening in another task, nothing
                    // more to do here.
                }
            }
            Err(e) => error!("can not convert event to string: {}", e),
        }
    }
}

pub(crate) type WsUsers = Arc<RwLock<HashMap<usize, WsUser>>>;

pub(crate) async fn user_connected<B: StorageBackend>(
    ws: WebSocket,
    storage: ResourceHandle<B>,
    tangle: ResourceHandle<MsTangle<B>>,
    users: WsUsers,
    node_id: String,
    auth_config: DashboardAuthConfig,
) {
    // Use a counter to assign a new unique ID for this user.
    let user_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    debug!("New ws user: {}.", user_id);

    // Split the socket into a sender and receive of messages.
    let (ws_tx, mut ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let receiver = ShutdownStream::new(shutdown_rx, UnboundedReceiverStream::new(rx));

    let task = tokio::task::spawn(receiver.forward(ws_tx).map(|result| {
        if let Err(e) = result {
            error!("websocket send error: {}", e);
        }
    }));

    // Save the sender in our list of connected users.
    users.write().await.insert(
        user_id,
        WsUser {
            tx,
            shutdown: Some(shutdown_tx),
            topics: HashSet::new(),
        },
    );

    // Handle incoming messages from the user
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("websocket error(uid={}): {}", user_id, e);
                break;
            }
        };
        user_message(user_id, msg, &users, &tangle, &storage, &node_id, &auth_config).await;
    }

    // ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(user_id, &users).await;

    let _ = task.await;
}

async fn user_message<B: StorageBackend>(
    user_id: usize,
    msg: Message,
    users: &WsUsers,
    tangle: &MsTangle<B>,
    storage: &B,
    node_id: &str,
    auth_config: &DashboardAuthConfig,
) {
    if !msg.is_binary() {
        return;
    }

    let bytes = msg.as_bytes();

    if bytes.len() < 2 {
        return;
    }

    let command: WsCommand = match bytes[0].try_into() {
        Ok(command) => command,
        Err(e) => {
            error!("Unknown websocket command: {}.", e);
            return;
        }
    };
    let topic: WsTopic = match bytes[1].try_into() {
        Ok(topic) => topic,
        Err(e) => {
            error!("Unknown websocket topic: {}.", e);
            return;
        }
    };

    if let Some(user) = users.write().await.get_mut(&user_id) {
        match command {
            WsCommand::Register => {
                if !topic.is_public() {
                    if bytes.len() < 3 {
                        return;
                    }
                    let jwt = JsonWebToken::from(match String::from_utf8(bytes[2..].to_vec()) {
                        Ok(jwt) => jwt,
                        Err(e) => {
                            error!("Invalid provided JWT: {}", e);
                            return;
                        }
                    });
                    if !jwt.validate(
                        node_id.to_owned(),
                        auth_config.user().to_owned(),
                        AUDIENCE_CLAIM.to_owned(),
                        b"secret",
                    ) {
                        return;
                    }
                }
                send_init_values(&topic, &user, tangle, storage).await;
                let _ = user.topics.insert(topic);
            }
            WsCommand::Unregister => {
                let _ = user.topics.remove(&topic);
            }
        }
    }
}

async fn user_disconnected(user_id: usize, users: &WsUsers) {
    debug!("User {} disconnected.", user_id);
    users.write().await.remove(&user_id);
}

async fn send_init_values<B: StorageBackend>(topic: &WsTopic, user: &WsUser, tangle: &MsTangle<B>, storage: &B) {
    match topic {
        WsTopic::SyncStatus => {
            let event = WsEvent::new(
                WsTopic::SyncStatus,
                WsEventInner::SyncStatus(SyncStatusResponse {
                    lmi: *tangle.get_latest_milestone_index(),
                    cmi: *tangle.get_confirmed_milestone_index(),
                }),
            );
            user.send(event);
        }
        WsTopic::DatabaseSizeMetrics => {
            let event = WsEvent::new(
                WsTopic::DatabaseSizeMetrics,
                WsEventInner::DatabaseSizeMetrics(DatabaseSizeMetricsResponse {
                    total: storage.size().await.unwrap().unwrap() as u64,
                    ts: 0,
                }),
            );
            user.send(event);
        }
        _ => {}
    }
}
