// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod commands;
pub(crate) mod responses;
mod topics;

use crate::{plugins::dashboard::websocket::responses::WsEvent, storage::StorageBackend};
use commands::WsCommand;
use topics::WsTopic;

use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use futures::{FutureExt, StreamExt};
use log::{debug, error};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

use crate::plugins::dashboard::{
    send_to_specific,
    websocket::responses::{
        database_size_metrics::DatabaseSizeMetricsResponse, sync_status::SyncStatusResponse, WsEventInner,
    },
};
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
    pub(crate) topics: HashSet<WsTopic>,
}

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
pub(crate) type WsUsers = Arc<RwLock<HashMap<usize, WsUser>>>;

pub(crate) async fn user_connected<B: StorageBackend>(
    ws: WebSocket,
    users: WsUsers,
    tangle: ResourceHandle<MsTangle<B>>,
    storage: ResourceHandle<B>,
) {
    // Use a counter to assign a new unique ID for this user.
    let user_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    debug!("New ws user: {}.", user_id);

    // Split the socket into a sender and receive of messages.
    let (ws_tx, mut ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);
    tokio::task::spawn(rx.forward(ws_tx).map(|result| {
        if let Err(e) = result {
            error!("websocket send error: {}", e);
        }
    }));

    // Save the sender in our list of connected users.
    users.write().await.insert(
        user_id,
        WsUser {
            tx,
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
        user_message(user_id, msg, &users, &tangle, &storage).await;
    }

    // ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(user_id, &users).await;
}

async fn user_message<B: StorageBackend>(
    user_id: usize,
    msg: Message,
    users: &WsUsers,
    tangle: &MsTangle<B>,
    storage: &B,
) {
    if msg.is_binary() {
        let bytes = msg.as_bytes();
        if bytes.len() >= 2 {
            let command = match bytes[0].try_into() {
                Ok(command) => command,
                Err(e) => {
                    error!("Unknown websocket command: {}.", e);
                    return;
                }
            };
            let topic = match bytes[1].try_into() {
                Ok(topic) => topic,
                Err(e) => {
                    error!("Unknown websocket topic: {}.", e);
                    return;
                }
            };

            if let Some(user) = users.write().await.get_mut(&user_id) {
                match command {
                    WsCommand::Register => {
                        send_init_values_for_topics(&topic, user_id, users, tangle, storage).await;
                        let _ = user.topics.insert(topic);
                    }
                    WsCommand::Unregister => {
                        let _ = user.topics.remove(&topic);
                    }
                }
            }
        }
    }
}

async fn user_disconnected(user_id: usize, users: &WsUsers) {
    debug!("User {} disconnected.", user_id);
    // Stream closed up, so remove from the user list
    users.write().await.remove(&user_id);
}

async fn send_init_values_for_topics<B: StorageBackend>(
    topic: &WsTopic,
    user_id: usize,
    users: &WsUsers,
    tangle: &MsTangle<B>,
    storage: &B,
) {
    match topic {
        &WsTopic::SyncStatus => {
            let event = WsEvent::new(
                WsTopic::SyncStatus,
                WsEventInner::SyncStatus(SyncStatusResponse {
                    lmi: *tangle.get_latest_milestone_index(),
                    lsmi: *tangle.get_latest_solid_milestone_index(),
                }),
            );
            send_to_specific(event, user_id, users).await;
        }
        &WsTopic::DatabaseSizeMetrics => {
            let event = WsEvent::new(
                WsTopic::DatabaseSizeMetrics,
                WsEventInner::DatabaseSizeMetrics(DatabaseSizeMetricsResponse {
                    total: storage.size().await.unwrap().unwrap() as u64,
                    ts: 0,
                }),
            );
            send_to_specific(event, user_id, users).await;
        }
        _ => {}
    }
}
