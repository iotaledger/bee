// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{filters::with_args, storage::StorageBackend, ApiArgs},
    types::{body::SuccessBody, dtos::PeerDto, responses::PeersResponse},
};

use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use std::{convert::Infallible, sync::Arc};

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("peers")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: Arc<ApiArgs<B>>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(peers)
        .boxed()
}

pub(crate) async fn peers<B: StorageBackend>(args: Arc<ApiArgs<B>>) -> Result<impl Reply, Infallible> {
    let mut peers_dtos = Vec::new();
    for peer in args.peer_manager.get_all().await {
        peers_dtos.push(PeerDto::from(peer.as_ref()));
    }
    Ok(warp::reply::json(&SuccessBody::new(PeersResponse(peers_dtos))))
}
