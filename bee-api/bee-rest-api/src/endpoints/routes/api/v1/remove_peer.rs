// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    filters::with_args, path_params::peer_id, rejection::CustomRejection, storage::StorageBackend, ApiArgsFullNode,
};

use bee_gossip::{Command::RemovePeer, PeerId};

use warp::{filters::BoxedFilter, http::StatusCode, reject, Filter, Rejection, Reply};

fn path() -> impl Filter<Extract = (PeerId,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("peers"))
        .and(peer_id())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::delete())
        .and(with_args(args))
        .and_then(|peer_id, args| async move { remove_peer(peer_id, args) })
        .boxed()
}

pub(crate) fn remove_peer<B: StorageBackend>(
    peer_id: PeerId,
    args: ApiArgsFullNode<B>,
) -> Result<impl Reply, Rejection> {
    if let Err(e) = args.network_command_sender.send(RemovePeer { peer_id }) {
        return Err(reject::custom(CustomRejection::NotFound(format!(
            "failed to remove peer: {}",
            e
        ))));
    }
    Ok(StatusCode::NO_CONTENT)
}
