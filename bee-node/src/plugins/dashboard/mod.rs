// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod config;

mod asset;
mod websocket;

use crate::storage::StorageBackend;

use config::DashboardConfig;
use websocket::{
    responses::{milestone, milestone_info, mps_metrics_updated, solid_info, sync_status, WsEvent},
    user_connected, WsUsers,
};

use bee_protocol::event::{LatestMilestoneChanged, MessageSolidified, MpsMetricsUpdated};
use bee_rest_api::config::RestApiConfig;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::MsTangle;

use asset::Asset;
use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{debug, error, info, warn};
use tokio::sync::mpsc;
use warp::{http::header::HeaderValue, path::FullPath, reply::Response, ws::Message, Filter, Rejection, Reply};
use warp_reverse_proxy::reverse_proxy_filter;

use std::{
    any::Any,
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

        let mut receiver = ShutdownStream::new(shutdown, rx);

        while let Some(event) = receiver.next().await {
            broadcast(f(event), &users).await;
        }

        debug!("Ws {} topic handler stopped.", topic);
    });

    bus.add_listener::<Dashboard, E, _>(move |event: &E| {
        if tx.send((*event).clone()).is_err() {
            warn!("Sending event to ws {} topic handler failed.", topic);
        }
    });
}

#[async_trait]
impl<N: Node> Worker<N> for Dashboard
where
    N::Backend: StorageBackend,
{
    type Config = (DashboardConfig, RestApiConfig);
    type Error = Infallible;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let dashboard_cfg = config.0;
        let rest_api_cfg = config.1;

        let tangle = node.resource::<MsTangle<N::Backend>>();

        // Keep track of all connected users, key is usize, value
        // is a websocket sender.
        let users = WsUsers::default();

        // Register event handlers
        topic_handler(node, "SyncStatus", &users, move |event: LatestMilestoneChanged| {
            sync_status::forward(event, &tangle)
        });
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
                        "http://".to_owned() + &rest_api_cfg.binding_socket_addr().to_string() + "/",
                    )
                    .map(|res| res),
                ));

            info!("Dashboard available at http://localhost:{}.", dashboard_cfg.port());

            let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(
                // TODO the whole address needs to be a config
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), dashboard_cfg.port()),
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
