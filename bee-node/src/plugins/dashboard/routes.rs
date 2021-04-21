// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    plugins::dashboard::{
        asset::Asset,
        auth::{auth, AUDIENCE_CLAIM},
        config::DashboardAuthConfig,
        rejection::CustomRejection,
        websocket::{user_connected, WsUsers},
    },
    storage::StorageBackend,
};

use bee_common::auth::jwt::JsonWebToken;
use bee_rest_api::endpoints::config::RestApiConfig;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

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
    let mut res = Response::new(asset.into());

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
        .or(warp::path!("peers" / ..).and_then(serve_index))
        .or(warp::path!("explorer" / ..).and_then(serve_index))
        .or(warp::path!("visualizer" / ..).and_then(serve_index))
        .or(warp::path!("settings" / ..).and_then(serve_index))
        .or(warp::path!("login" / ..).and_then(serve_index))
}

pub(crate) fn ws_routes<B: StorageBackend>(
    storage: ResourceHandle<B>,
    tangle: ResourceHandle<MsTangle<B>>,
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
    node_id: String,
    auth_config: DashboardAuthConfig,
    rest_api_config: RestApiConfig,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let allowed_routes = warp::get()
        .and(warp::path!("api" / "v1" / "info" / ..))
        .or(warp::get().and(warp::path!("api" / "v1" / "messages" / ..)))
        .or(warp::get().and(warp::path!("api" / "v1" / "outputs" / ..)))
        .or(warp::get().and(warp::path!("api" / "v1" / "addresses" / ..)))
        .or(warp::get().and(warp::path!("api" / "v1" / "milestones" / ..)))
        .or(auth_filter(node_id.clone(), auth_config.clone())
            .and(warp::get())
            .and(warp::path!("api" / "v1" / "peers" / ..)))
        .or(auth_filter(node_id.clone(), auth_config.clone())
            .and(warp::post())
            .and(warp::path!("api" / "v1" / "peers" / ..)))
        .or(auth_filter(node_id, auth_config)
            .and(warp::delete())
            .and(warp::path!("api" / "v1" / "peers" / ..)));

    allowed_routes
        .and(reverse_proxy_filter(
            "".to_string(),
            "http://localhost:".to_owned() + &rest_api_config.binding_socket_addr().port().to_string() + "/",
        ))
        .map(|_, res| res)
}

pub fn auth_filter(
    node_id: String,
    auth_config: DashboardAuthConfig,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    let node_id_filter = warp::any().map(move || node_id.clone());
    let auth_config_filter = warp::any().map(move || auth_config.clone());

    headers_cloned()
        .map(move |headers: HeaderMap<HeaderValue>| (headers))
        .and(node_id_filter)
        .and(auth_config_filter)
        .and_then(
            move |headers: HeaderMap<HeaderValue>, node_id: String, auth_config: DashboardAuthConfig| async move {
                let header = match headers.get(AUTHORIZATION) {
                    Some(v) => v,
                    None => return Err(reject::custom(CustomRejection::Forbidden)),
                };
                let auth_header = match std::str::from_utf8(header.as_bytes()) {
                    Ok(v) => v,
                    Err(_) => return Err(reject::custom(CustomRejection::Forbidden)),
                };
                if !auth_header.starts_with(BEARER) {
                    return Err(reject::custom(CustomRejection::Forbidden));
                }

                let jwt = JsonWebToken::from(auth_header.trim_start_matches(BEARER).to_owned());

                if jwt
                    .validate(
                        node_id.clone(),
                        auth_config.user().to_owned(),
                        AUDIENCE_CLAIM.to_owned(),
                        b"secret",
                    )
                    .is_err()
                {
                    return Err(reject::custom(CustomRejection::Forbidden));
                }

                Ok(())
            },
        )
        .untuple_one()
}

pub(crate) fn auth_route(
    node_id: String,
    auth_config: DashboardAuthConfig,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let node_id_filter = warp::any().map(move || node_id.clone());
    let auth_config_filter = warp::any().map(move || auth_config.clone());

    warp::post()
        .and(warp::path("auth"))
        .and(node_id_filter)
        .and(auth_config_filter)
        .and(warp::body::json())
        .and_then(auth)
}

pub(crate) fn routes<B: StorageBackend>(
    storage: ResourceHandle<B>,
    tangle: ResourceHandle<MsTangle<B>>,
    node_id: String,
    auth_config: DashboardAuthConfig,
    rest_api_config: RestApiConfig,
    users: WsUsers,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    index_filter()
        .or(asset_routes())
        .or(page_routes())
        .or(ws_routes(storage, tangle, users, node_id.clone(), auth_config.clone()))
        .or(api_routes(node_id.clone(), auth_config.clone(), rest_api_config))
        .or(auth_route(node_id, auth_config))
}
