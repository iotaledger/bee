// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        filters::with_args, path_params::peer_id, rejection::CustomRejection, storage::StorageBackend, ApiArgs,
    },
    types::{body::SuccessBody, dtos::PeerDto, responses::PeerResponse},
};

use bee_gossip::PeerId;

use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::sync::Arc;

fn path() -> impl Filter<Extract = (PeerId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("peers"))
        .and(peer_id())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: Arc<ApiArgs<B>>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(peer)
        .boxed()
}

pub(crate) async fn peer<B: StorageBackend>(peer_id: PeerId, args: Arc<ApiArgs<B>>) -> Result<impl Reply, Rejection> {
    match args.peer_manager.get(&peer_id).await {
        Some(peer_entry) => Ok(warp::reply::json(&SuccessBody::new(PeerResponse(PeerDto::from(
            peer_entry.0.as_ref(),
        ))))),
        None => Err(reject::custom(CustomRejection::NotFound("peer not found".to_string()))),
    }
}
