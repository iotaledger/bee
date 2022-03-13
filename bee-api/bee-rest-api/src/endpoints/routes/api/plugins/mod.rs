// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod debug;

use crate::endpoints::{config::RestApiConfig, storage::StorageBackend};

use bee_protocol::workers::{MessageRequesterWorker, RequestedMessages};
use bee_runtime::{event::Bus, resource::ResourceHandle};
use bee_tangle::Tangle;

use warp::{self, Filter, Rejection, Reply};

use std::net::IpAddr;


pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().nest("plugins", debug::filter())
}
