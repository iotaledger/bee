// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod config;

mod asset;
mod websocket;

use config::DashboardConfig;
use websocket::{
    responses::{milestone, milestone_info, mps_metrics_updated, solid_info, sync_status, WsEvent},
    user_connected, WsUsers,
};

use bee_common_pt2::{node::Node, worker::Worker};
use bee_protocol::tangle::MsTangle;

use asset::Asset;
use async_trait::async_trait;
use log::{debug, error, info};
use warp::{http::header::HeaderValue, path::FullPath, reply::Response, ws::Message, Filter, Rejection, Reply};

use std::{
    convert::Infallible,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

#[derive(Default)]
pub struct Dashboard {}

#[async_trait]
impl<N: Node> Worker<N> for Dashboard {
    type Config = DashboardConfig;
    type Error = Infallible;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let bus = node.bus();
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            // Keep track of all connected users, key is usize, value
            // is a websocket sender.
            let users = WsUsers::default();

            // Register event handlers
            sync_status::register(bus.clone(), users.clone(), tangle.clone());
            mps_metrics_updated::register(bus.clone(), users.clone());
            milestone::register(bus.clone(), users.clone());
            solid_info::register(bus.clone(), users.clone());
            milestone_info::register(bus.clone(), users.clone());

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
                    }));

            info!("Dashboard available at http://localhost:{}.", config.port());

            let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), config.port()),
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

pub(crate) async fn broadcast(event: WsEvent, users: WsUsers) {
    match serde_json::to_string(&event) {
        Ok(event_as_string) => {
            for (_uid, user) in users.read().await.iter() {
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
