// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    plugins::dashboard::{
        asset::Asset,
        auth::{auth},
        config::DashboardAuthConfig,
        rejection::CustomRejection,
        websocket::{user_connected, WsUsers},
    },
    storage::NodeStorageBackend,
    Local,
};

use bee_rest_api::endpoints::config::RestApiConfig;
use bee_rest_api::endpoints::permission::DASHBOARD_AUDIENCE_CLAIM;

use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;

use auth_helper::jwt::JsonWebToken;
use log::debug;
use warp::{
    filters::header::headers_cloned,
    http::header::{HeaderMap, HeaderValue, AUTHORIZATION},
    path::FullPath,
    reject,
    reply::Response,
    Filter, Rejection, Reply,
};
use warp_reverse_proxy::reverse_proxy_filter;

const BEARER: &str = "Bearer ";

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
    let mut res = Response::new(asset.data.into());

    res.headers_mut()
        .insert("content-type", HeaderValue::from_str(mime.as_ref()).unwrap());

    Ok(res)
}

pub(crate) fn index_filter() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path::end().and_then(serve_index)
}

pub(crate) fn asset_routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("branding")
        .and(warp::path::full())
        .and_then(serve_full_path)
        .or(warp::path("static").and(warp::path::full()).and_then(serve_full_path))
}

pub(crate) fn page_routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("analytics" / ..)
        .and_then(serve_index)
        .or(warp::path!("dashboard" / ..).and_then(serve_index))
        .or(warp::path!("explorer" / ..).and_then(serve_index))
        .or(warp::path!("login" / ..).and_then(serve_index))
        .or(warp::path!("peers" / ..).and_then(serve_index))
        .or(warp::path!("settings" / ..).and_then(serve_index))
        .or(warp::path!("visualizer" / ..).and_then(serve_index))
}

pub(crate) fn ws_routes<S: NodeStorageBackend>(
    storage: ResourceHandle<S>,
    tangle: ResourceHandle<Tangle<S>>,
    users: WsUsers,
    node_id: String,
    auth_config: DashboardAuthConfig,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let storage_filter = warp::any().map(move || storage.clone());
    let tangle_filter = warp::any().map(move || tangle.clone());
    let users_filter = warp::any().map(move || users.clone());
    let node_id_filter = warp::any().map(move || node_id.clone());
    let auth_config_filter = warp::any().map(move || auth_config.clone());

    warp::path("ws")
        .and(warp::ws())
        .and(storage_filter)
        .and(tangle_filter)
        .and(users_filter)
        .and(node_id_filter)
        .and(auth_config_filter)
        .map(|ws: warp::ws::Ws, storage, tangle, users, node_id, auth_config| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, storage, tangle, users, node_id, auth_config))
        })
}

pub(crate) fn api_routes(
    rest_api_config: RestApiConfig,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {

    let allowed_routes = warp::get()
        .and(warp::path!("api" / "v1" / "info" / ..))
        .or(warp::get().and(warp::path!("api" / "v1" / "messages" / ..)))
        .or(warp::get().and(warp::path!("api" / "v1" / "outputs" / ..)))
        .or(warp::get().and(warp::path!("api" / "v1" / "addresses" / ..)))
        .or(warp::get().and(warp::path!("api" / "v1" / "milestones" / ..)))
        .or(warp::get().and(warp::path!("api" / "v1" / "peers" / ..)))
        .or(warp::post().and(warp::path!("api" / "v1" / "peers" / ..)))
        .or(warp::delete().and(warp::path!("api" / "v1" / "peers" / ..)));

    allowed_routes
        .and(reverse_proxy_filter(
            "".to_string(),
            "http://".to_owned() + &rest_api_config.bind_socket_addr().to_string() + "/",
        ))
        .map(|_, res| res)
}

pub(crate) fn auth_route(
    local: Local,
    auth_config: DashboardAuthConfig,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let local_filter = warp::any().map(move || local.clone());
    let auth_config_filter = warp::any().map(move || auth_config.clone());

    warp::post()
        .and(warp::path("auth"))
        .and(local_filter)
        .and(auth_config_filter)
        .and(warp::body::json())
        .and_then(auth)
}

pub(crate) fn routes<S: NodeStorageBackend>(
    storage: ResourceHandle<S>,
    tangle: ResourceHandle<Tangle<S>>,
    local: Local,
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
            local.peer_id().to_string(),
            auth_config.clone(),
        ))
        .or(api_routes( rest_api_config))
        .or(auth_route(local, auth_config))
}
