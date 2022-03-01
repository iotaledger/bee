// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Dashboard plugin for the Bee node.

#![warn(missing_docs)]

/// Dashboard configuration.
pub mod config;

mod asset;
mod auth;
mod rejection;
mod routes;
mod storage;
mod websocket;
mod workers;

use crate::{
    config::DashboardConfig,
    storage::StorageBackend,
    websocket::{
        responses::{milestone, milestone_info, sync_status, WsEvent},
        WsUsers,
    },
    workers::{
        confirmed_ms_metrics::confirmed_ms_metrics_worker, db_size_metrics::db_size_metrics_worker,
        node_status::node_status_worker, peer_metric::peer_metric_worker,
    },
};

use bee_ledger::workers::event::MilestoneConfirmed;
use bee_protocol::workers::{
    event::{MessageSolidified, MpsMetricsUpdated, TipAdded, TipRemoved, VertexCreated},
    MetricsWorker, PeerManagerResWorker,
};
use bee_rest_api::endpoints::config::RestApiConfig;
use bee_runtime::{
    node::{Node, NodeBuilder},
    shutdown_stream::ShutdownStream,
    worker::Worker,
};
use bee_tangle::{event::LatestMilestoneChanged, Tangle, TangleWorker};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::Message;

use std::{
    any::{Any, TypeId},
    convert::Infallible,
};

pub(crate) type Bech32Hrp = String;
pub(crate) type NodeAlias = String;
pub(crate) type NodeId = String;

const CONFIRMED_THRESHOLD: u32 = 5;

/// Initializes the dashboard plugin.
pub fn init<N: Node>(
    dashboard_config: DashboardConfig,
    rest_api_config: RestApiConfig,
    node_id: NodeId,
    node_alias: NodeAlias,
    bech32_hrp: Bech32Hrp,
    node_builder: N::Builder,
) -> N::Builder
where
    N::Backend: StorageBackend,
{
    node_builder.with_worker_cfg::<DashboardPlugin>((dashboard_config, rest_api_config, node_id, node_alias, bech32_hrp))
}

fn topic_handler<N, E, F>(node: &mut N, topic: &'static str, users: &WsUsers, require_node_confirmed: bool, f: F)
where
    N: Node,
    N::Backend: StorageBackend,
    E: Any + Clone + Send + Sync,
    F: 'static + Fn(E) -> WsEvent + Send + Sync,
{
    let tangle = node.resource::<Tangle<N::Backend>>();
    let bus = node.bus();
    let users = users.clone();
    let (tx, rx) = mpsc::unbounded_channel();

    node.spawn::<DashboardPlugin, _, _>(|shutdown| async move {
        debug!("Ws {} topic handler running.", topic);

        let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

        while let Some(event) = receiver.next().await {
            if require_node_confirmed {
                if tangle.is_confirmed_threshold(CONFIRMED_THRESHOLD) {
                    broadcast(f(event), &users).await;
                }
            } else {
                broadcast(f(event), &users).await;
            }
        }

        debug!("Ws {} topic handler stopped.", topic);
    });

    bus.add_listener::<DashboardPlugin, E, _>(move |event: &E| {
        // The lifetime of the listeners is tied to the lifetime of the Dashboard worker so they are removed together.
        // However, topic handlers are shutdown as soon as the signal is received, causing this send to potentially
        // fail and spam the output. The return is then ignored as not being essential.
        let _ = tx.send((*event).clone());
    });
}

/// Dashboard plugin.
#[derive(Default)]
pub struct DashboardPlugin;

#[async_trait]
impl<N: Node> Worker<N> for DashboardPlugin
where
    N::Backend: StorageBackend,
{
    type Config = (DashboardConfig, RestApiConfig, NodeId, NodeAlias, Bech32Hrp);
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<MetricsWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        // TODO: load them differently if possible
        let (config, rest_api_config, node_id, node_alias, bech32_hrp) = config;
        let tangle = node.resource::<Tangle<N::Backend>>();
        let storage = node.storage();

        // Keep track of all connected users, key is usize, value is a websocket sender.
        let users = WsUsers::default();

        // Register event handlers
        {
            let tangle = tangle.clone();
            topic_handler(
                node,
                "SyncStatus",
                &users,
                false,
                move |event: LatestMilestoneChanged| sync_status::forward_latest_milestone_changed(event, &tangle),
            );
        }
        {
            let tangle = tangle.clone();
            topic_handler(node, "SyncStatus", &users, false, move |event: MilestoneConfirmed| {
                sync_status::forward_confirmed_milestone_changed(&event, &tangle)
            });
        }
        topic_handler(
            node,
            "MpsMetricsUpdated",
            &users,
            false,
            <WsEvent as From<MpsMetricsUpdated>>::from,
        );
        topic_handler(node, "Milestone", &users, false, milestone::forward);
        topic_handler(
            node,
            "SolidInfo",
            &users,
            true,
            <WsEvent as From<MessageSolidified>>::from,
        );
        topic_handler(node, "MilestoneInfo", &users, false, milestone_info::forward);
        topic_handler(node, "Vertex", &users, true, <WsEvent as From<VertexCreated>>::from);
        topic_handler(
            node,
            "MilestoneConfirmed",
            &users,
            false,
            <WsEvent as From<MilestoneConfirmed>>::from,
        );
        topic_handler(node, "TipInfo", &users, true, <WsEvent as From<TipAdded>>::from);
        topic_handler(node, "TipInfo", &users, true, <WsEvent as From<TipRemoved>>::from);

        // run sub-workers
        confirmed_ms_metrics_worker(node, &users);
        db_size_metrics_worker(node, &users);
        node_status_worker(node, node_id.clone(), node_alias, bech32_hrp, &users);
        peer_metric_worker(node, &users);

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let routes = routes::routes(
                storage.clone(),
                tangle.clone(),
                node_id.clone(),
                config.auth().clone(),
                rest_api_config.clone(),
                users.clone(),
            );

            let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(config.bind_socket_addr(), async {
                shutdown.await.ok();
            });

            info!("Dashboard available at http://{}.", config.bind_socket_addr());

            server.await;

            let mut readies = Vec::new();

            for (_, user) in users.write().await.iter_mut() {
                if let Some(shutdown) = user.shutdown.take() {
                    let _ = shutdown.send(());
                    readies.push(user.shutdown_ready.take().unwrap());
                }
            }

            futures::future::join_all(readies).await;

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}

pub(crate) async fn broadcast(event: WsEvent, users: &WsUsers) {
    match serde_json::to_string(&event) {
        Ok(as_text) => {
            for (_, user) in users.read().await.iter() {
                if user.topics.contains(&event.kind) {
                    if let Err(_disconnected) = user.tx.send(Ok(Message::text(as_text.clone()))) {
                        // The tx is disconnected, our `user_disconnected` code should be happening in another task,
                        // nothing more to do here.
                    }
                }
            }
        }
        Err(e) => error!("can not convert event to string: {}", e),
    }
}
