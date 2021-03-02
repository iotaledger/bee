// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::StorageBackend;

use bee_protocol::PeerManager;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use warp::{http::StatusCode, Reply};

use std::{
    convert::Infallible,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) async fn health<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
    peer_manager: ResourceHandle<PeerManager>,
) -> Result<impl Reply, Infallible> {
    if is_healthy(&tangle, &peer_manager).await {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::SERVICE_UNAVAILABLE)
    }
}

pub async fn is_healthy<B: StorageBackend>(tangle: &MsTangle<B>, peer_manager: &PeerManager) -> bool {
    if !tangle.is_synced() {
        return false;
    }

    if peer_manager.connected_peers().await == 0 {
        return false;
    }

    match tangle
        .get_milestone_message_id(tangle.get_latest_milestone_index())
        .await
    {
        Some(message_id) => match tangle.get_metadata(&message_id).await {
            Some(metadata) => {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Clock may have gone backwards")
                    .as_millis() as u64;
                let latest_milestone_arrival_timestamp = metadata.arrival_timestamp();
                if current_time - latest_milestone_arrival_timestamp > 5 * 60 * 60000 {
                    return false;
                }
            }
            None => return false,
        },
        None => return false,
    }

    true
}
