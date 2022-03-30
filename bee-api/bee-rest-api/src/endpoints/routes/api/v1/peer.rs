// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_gossip::PeerId;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        filters::with_args, path_params::peer_id, rejection::CustomRejection, storage::StorageBackend, ApiArgsFullNode,
    },
    types::{body::SuccessBody, dtos::PeerDto, responses::PeerResponse},
};

fn path() -> impl Filter<Extract = (PeerId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("peers"))
        .and(peer_id())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|peer_id, args| async move { peer(peer_id, args) })
        .boxed()
}

pub(crate) fn peer<B: StorageBackend>(peer_id: PeerId, args: ApiArgsFullNode<B>) -> Result<impl Reply, Rejection> {
    args.peer_manager
        .get_map(&peer_id, |peer_entry| {
            Ok(warp::reply::json(&SuccessBody::new(PeerResponse(PeerDto::from(
                peer_entry.0.as_ref(),
            )))))
        })
        .unwrap_or_else(|| Err(reject::custom(CustomRejection::NotFound("peer not found".to_string()))))
}
