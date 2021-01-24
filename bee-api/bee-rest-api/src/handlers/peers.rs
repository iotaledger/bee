// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::RestApiConfig,
    constants::{BEE_GIT_COMMIT, BEE_VERSION},
    handlers::{BodyInner, SuccessBody},
    storage::StorageBackend,
    Bech32Hrp, NetworkId,
};

use bee_protocol::config::ProtocolConfig;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use serde::Serialize;
use warp::Reply;

use std::convert::Infallible;

pub(crate) async fn peers<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&SuccessBody::new(PeersResponse(Vec::new()))))
}

/// Response of GET /api/v1/info
#[derive(Clone, Debug, Serialize)]
pub struct PeersResponse(pub Vec<Peers>);

impl BodyInner for PeersResponse {}
