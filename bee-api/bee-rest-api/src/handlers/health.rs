// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::Backend;
use bee_common::node::ResHandle;
use bee_protocol::tangle::MsTangle;
use std::{
    convert::Infallible,
    time::{SystemTime, UNIX_EPOCH},
};
use warp::{http::StatusCode, Reply};

pub(crate) async fn health<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> Result<impl Reply, Infallible> {
    if is_healthy(tangle).await {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::SERVICE_UNAVAILABLE)
    }
}

pub(crate) async fn is_healthy<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> bool {
    if !tangle.is_synced() {
        return false;
    }

    // TODO: check if number of peers != 0 else return false

    match tangle.get_milestone_message_id(tangle.get_latest_milestone_index()) {
        Some(message_id) => match tangle.get_metadata(&message_id) {
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
