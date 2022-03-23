// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::convert::Infallible;

use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use crate::{
    endpoints::{filters::with_args, storage::StorageBackend, ApiArgsFullNode},
    types::{body::SuccessBody, dtos::PeerDto, responses::PeersResponse},
};

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("peers")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|args| async move { peers(args) })
        .boxed()
}

pub(crate) fn peers<B: StorageBackend>(args: ApiArgsFullNode<B>) -> Result<impl Reply, Infallible> {
    let mut peers_dtos = Vec::new();
    for peer in args.peer_manager.get_all() {
        peers_dtos.push(PeerDto::from(peer.as_ref()));
    }
    Ok(warp::reply::json(&SuccessBody::new(PeersResponse(peers_dtos))))
}
