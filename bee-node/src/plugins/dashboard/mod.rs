// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod config;

mod asset;
mod websocket;
mod workers;

use crate::{
    config::NodeConfig,
    plugins::dashboard::{
        config::DashboardConfig,
        websocket::{
            responses::{
                confirmed_info, milestone, milestone_info, mps_metrics_updated, solid_info, sync_status, tip_info,
                vertex, WsEvent,
            },
            user_connected, WsUsers,
        },
        workers::{
            confirmed_ms_metrics::confirmed_ms_metrics_worker, db_size_metrics::db_size_metrics_worker,
            node_status::node_status_worker, peer_metric::peer_metric_worker,
        },
    },
    storage::StorageBackend,
};

use bee_ledger::event::MilestoneConfirmed;
use bee_protocol::{
    event::{
        LatestMilestoneChanged, LatestSolidMilestoneChanged, MessageSolidified, MpsMetricsUpdated, NewVertex, TipAdded,
        TipRemoved,
    },
    MetricsWorker, TangleWorker,
};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::MsTangle;

use asset::Asset;
use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{http::header::HeaderValue, path::FullPath, reply::Response, ws::Message, Filter, Rejection, Reply};
use warp_reverse_proxy::reverse_proxy_filter;

use std::{
    any::{Any, TypeId},
    convert::Infallible,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

#[derive(Default)]
pub struct Dashboard {}

fn topic_handler<N, E, F>(node: &mut N, topic: &'static str, users: &WsUsers, f: F)
where
    N: Node,
    N::Backend: StorageBackend,
    E: Any + Clone + Send + Sync,
    F: 'static + Fn(E) -> WsEvent + Send + Sync,
{
    let bus = node.bus();
    let users = users.clone();
    let (tx, rx) = mpsc::unbounded_channel();

    node.spawn::<Dashboard, _, _>(|shutdown| async move {
        debug!("Ws {} topic handler running.", topic);

        let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

        while let Some(event) = receiver.next().await {
            broadcast(f(event), &users).await;
        }

        debug!("Ws {} topic handler stopped.", topic);
    });

    bus.add_listener::<Dashboard, E, _>(move |event: &E| {
        // The lifetime of the listeners is tied to the lifetime of the Dashboard worker so they are removed together.
        // However, topic handlers are shutdown as soon as the signal is received, causing this send to potentially
        // fail and spam the output. The return is then ignored as not being essential.
        let _ = tx.send((*event).clone());
    });
}

#[async_trait]
impl<N: Node> Worker<N> for Dashboard
where
    N::Backend: StorageBackend,
{
    type Config = DashboardConfig;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>(), TypeId::of::<MetricsWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        // TODO: load them differently if possible
        let node_config = node.resource::<NodeConfig<N::Backend>>();
        let rest_api_config = node_config.rest_api.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();

        // Keep track of all connected users, key is usize, value
        // is a websocket sender.
        let users = WsUsers::default();

        // Register event handlers
        {
            let tangle = tangle.clone();
            topic_handler(node, "SyncStatus", &users, move |event: LatestMilestoneChanged| {
                sync_status::forward_latest_milestone_changed(event, &tangle)
            });
        }
        {
            let tangle = tangle.clone();
            topic_handler(node, "SyncStatus", &users, move |event: LatestSolidMilestoneChanged| {
                sync_status::forward_solid_milestone_changed(event, &tangle)
            });
        }
        topic_handler(node, "MpsMetricsUpdated", &users, move |event: MpsMetricsUpdated| {
            mps_metrics_updated::forward(event)
        });
        topic_handler(node, "Milestone", &users, move |event: LatestMilestoneChanged| {
            milestone::forward(event)
        });
        topic_handler(node, "SolidInfo", &users, move |event: MessageSolidified| {
            solid_info::forward(event)
        });
        topic_handler(node, "MilestoneInfo", &users, move |event: LatestMilestoneChanged| {
            milestone_info::forward(event)
        });
        topic_handler(node, "Vertex", &users, move |event: NewVertex| vertex::forward(event));
        topic_handler(node, "MilestoneConfirmed", &users, move |event: MilestoneConfirmed| {
            confirmed_info::forward(event)
        });
        topic_handler(node, "TipInfo", &users, move |event: TipAdded| {
            tip_info::forward_tip_added(event)
        });
        topic_handler(node, "TipInfo", &users, move |event: TipRemoved| {
            tip_info::forward_tip_removed(event)
        });

        // run sub-workers
        confirmed_ms_metrics_worker(node, &users);
        db_size_metrics_worker(node, &users);
        node_status_worker(node, &users);
        peer_metric_worker(node, &users);

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            // Turn our "state" into a new Filter...
            let users = warp::any().map(move || users.clone());

            let routes = warp::path::end()
                .and_then(serve_index)
                .or(warp::path("branding").and(warp::path::full()).and_then(serve_full_path))
                .or(warp::path("static").and(warp::path::full()).and_then(serve_full_path))
                .or(warp::path("ws")
                    .and(warp::ws())
                    .and(users)
                    .map(|ws: warp::ws::Ws, users| {
                        // This will call our function if the handshake succeeds.
                        ws.on_upgrade(move |socket| user_connected(socket, users))
                    }))
                .or(warp::path!("api" / ..).and(
                    reverse_proxy_filter(
                        "".to_string(),
                        "http://".to_owned() + &rest_api_config.binding_socket_addr().to_string() + "/",
                    )
                    .map(|res| res),
                ))
                .or(warp::path!("explorer" / ..).and_then(serve_index));

            info!("Dashboard available at http://localhost:{}.", config.port());

            let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(
                // TODO the whole address needs to be a config
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), config.port()),
                async {
                    shutdown.await.ok();
                },
            );

            server.await;

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}

async fn serve_index() -> Result<impl Reply, Rejection> {
    serve_asset("index.html")
}

async fn serve_full_path(path: FullPath) -> Result<impl Reply, Rejection> {
    serve_asset(&path.as_str()[1..])
}

fn serve_asset(path: &str) -> Result<impl Reply, Rejection> {
    debug!("Serving asset {}...", path);

    let asset = Asset::get(path).ok_or_else(warp::reject::not_found)?;
    // TODO remove dep
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let mut res = Response::new(asset.into());

    res.headers_mut()
        .insert("content-type", HeaderValue::from_str(mime.as_ref()).unwrap());

    Ok(res)
}

pub(crate) async fn broadcast(event: WsEvent, users: &WsUsers) {
    match serde_json::to_string(&event) {
        Ok(event_as_string) => {
            for (_, user) in users.read().await.iter() {
                if user.topics.contains(&event.kind) {
                    if let Err(_disconnected) = user.tx.send(Ok(Message::text(event_as_string.clone()))) {
                        // The tx is disconnected, our `user_disconnected` code
                        // should be happening in another task, nothing more to
                        // do here.
                    }
                }
            }
        }
        Err(e) => error!("can not convert event to string: {}", e),
    }
}
