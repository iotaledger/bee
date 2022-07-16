// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_gossip::{Keypair, PeerId};
use bee_rest_api::config::RestApiConfig;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use log::debug;
use warp::{http::header::HeaderValue, path::FullPath, reply::Response, Filter, Rejection, Reply};
use warp_reverse_proxy::reverse_proxy_filter;

use crate::{
    asset::Asset,
    auth::auth,
    config::DashboardAuthConfig,
    storage::StorageBackend,
    websocket::{user_connected, WsUsers},
};

fn serve_index() -> Result<impl Reply, Rejection> {
    serve_asset("index.html")
}

fn serve_full_path(path: FullPath) -> Result<impl Reply, Rejection> {
    serve_asset(&path.as_str()[1..])
}

fn serve_asset(path: &str) -> Result<impl Reply, Rejection> {
    debug!("Serving asset {}...", path);

    let asset = Asset::get(path).ok_or_else(warp::reject::not_found)?;
    // TODO remove dep
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let mut res = Response::new(asset.data.into());

    res.headers_mut()
        .insert("content-type", HeaderValue::from_str(mime.as_ref()).unwrap());

    Ok(res)
}

pub(crate) fn index_filter() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path::end().and_then(|| async move { serve_index() })
}

pub(crate) fn asset_routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("branding")
        .and(warp::path::full())
        .and_then(|path| async move { serve_full_path(path) })
        .or(warp::path("static")
            .and(warp::path::full())
            .and_then(|path| async move { serve_full_path(path) }))
}

pub(crate) fn page_routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("analytics" / ..)
        .and_then(|| async { serve_index() })
        .or(warp::path!("dashboard" / ..).and_then(|| async { serve_index() }))
        .or(warp::path!("explorer" / ..).and_then(|| async { serve_index() }))
        .or(warp::path!("login" / ..).and_then(|| async { serve_index() }))
        .or(warp::path!("peers" / ..).and_then(|| async { serve_index() }))
        .or(warp::path!("settings" / ..).and_then(|| async { serve_index() }))
        .or(warp::path!("visualizer" / ..).and_then(|| async { serve_index() }))
}

pub(crate) fn ws_routes<S: StorageBackend>(
    storage: ResourceHandle<S>,
    tangle: ResourceHandle<Tangle<S>>,
    users: WsUsers,
    node_id: PeerId,
    node_keypair: Keypair,
    auth_config: DashboardAuthConfig,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let storage_filter = warp::any().map(move || storage.clone());
    let tangle_filter = warp::any().map(move || tangle.clone());
    let users_filter = warp::any().map(move || users.clone());
    let node_id_filter = warp::any().map(move || node_id);
    let node_keypair_filter = warp::any().map(move || node_keypair.clone());
    let auth_config_filter = warp::any().map(move || auth_config.clone());

    warp::path("ws")
        .and(warp::ws())
        .and(storage_filter)
        .and(tangle_filter)
        .and(users_filter)
        .and(node_id_filter)
        .and(node_keypair_filter)
        .and(auth_config_filter)
        .map(
            |ws: warp::ws::Ws, storage, tangle, users, node_id, node_keypair, auth_config| {
                // This will call our function if the handshake succeeds.
                ws.on_upgrade(move |socket| {
                    user_connected(socket, storage, tangle, users, node_id, node_keypair, auth_config)
                })
            },
        )
}

pub(crate) fn api_routes(
    rest_api_config: RestApiConfig,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("api" / ..)
        .and(reverse_proxy_filter(
            "".to_string(),
            "http://".to_owned() + &rest_api_config.bind_socket_addr().to_string() + "/",
        ))
        .map(|res| res)
}

pub(crate) fn auth_route(
    node_id: PeerId,
    node_keypair: Keypair,
    auth_config: DashboardAuthConfig,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let node_id_filter = warp::any().map(move || node_id);
    let node_keypair_filter = warp::any().map(move || node_keypair.clone());
    let auth_config_filter = warp::any().map(move || auth_config.clone());

    warp::post()
        .and(warp::path("auth"))
        .and(node_id_filter)
        .and(node_keypair_filter)
        .and(auth_config_filter)
        .and(warp::body::json())
        .and_then(|node_id, keypair, config, body| async move { auth(node_id, keypair, config, body) })
}

pub(crate) fn routes<S: StorageBackend>(
    storage: ResourceHandle<S>,
    tangle: ResourceHandle<Tangle<S>>,
    node_id: PeerId,
    node_keypair: Keypair,
    auth_config: DashboardAuthConfig,
    rest_api_config: RestApiConfig,
    users: WsUsers,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    index_filter()
        .or(asset_routes())
        .or(page_routes())
        .or(ws_routes(
            storage,
            tangle,
            users,
            node_id,
            node_keypair.clone(),
            auth_config.clone(),
        ))
        .or(api_routes(rest_api_config))
        .or(auth_route(node_id, node_keypair, auth_config))
}
