// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    handlers::{BodyInner, SuccessBody},
    storage::StorageBackend,
    types::PeerDto,
};

use bee_protocol::PeerManager;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use serde::Serialize;
use warp::Reply;

use std::convert::Infallible;

pub(crate) async fn peers<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
    peer_manager: ResourceHandle<PeerManager>,
) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&SuccessBody::new(PeersResponse(Vec::new()))))
}

/// Response of GET /api/v1/info
#[derive(Clone, Debug, Serialize)]
pub struct PeersResponse(pub Vec<PeerDto>);

impl BodyInner for PeersResponse {}
