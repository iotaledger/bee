// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod commands;
pub(crate) mod responses;
mod topics;

use commands::WsCommand;
use topics::WsTopic;

use futures::{FutureExt, StreamExt};
use log::{error, info};
use tokio::sync::{mpsc, RwLock};
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
    pub(crate) topics: HashSet<WsTopic>,
}

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
pub(crate) type WsUsers = Arc<RwLock<HashMap<usize, WsUser>>>;

pub(crate) async fn user_connected(ws: WebSocket, users: WsUsers) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    info!("new user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (ws_tx, mut ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.forward(ws_tx).map(|result| {
        info!("forwarded {:?}", result);
        if let Err(e) = result {
            error!("websocket send error: {}", e);
        }
    }));

    // Save the sender in our list of connected users.
    users.write().await.insert(
        my_id,
        WsUser {
            tx,
            topics: {
                let mut t = HashSet::new();
                t.insert(WsTopic::SyncStatus);
                t.insert(WsTopic::MPSMetrics);
                t.insert(WsTopic::Milestone);
                t.insert(WsTopic::SolidInfo);
                t.insert(WsTopic::MilestoneInfo);
                t
            },
        },
    );

    // Handle incoming messages from the user
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }

    // ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users).await;
}

async fn user_message(my_id: usize, msg: Message, users: &WsUsers) {
    if msg.is_binary() {
        let bytes = msg.as_bytes();
        if bytes.len() >= 2 {
            if let (Ok(command), Ok(topic)) = (bytes[0].try_into(), bytes[1].try_into()) {
                // unwrap is safe since the user is still present in WsUsers
                let mut locked_users = users.write().await;
                let user = locked_users.get_mut(&my_id).unwrap();
                match command {
                    WsCommand::Register => {
                        let _ = user.topics.insert(topic);
                    }
                    WsCommand::Unregister => {
                        let _ = user.topics.remove(&topic);
                    }
                }
            } else {
                error!("unknown command/topic");
            }
        }
    }
}

async fn user_disconnected(my_id: usize, users: &WsUsers) {
    info!("user disconnected: {}", my_id);
    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}
