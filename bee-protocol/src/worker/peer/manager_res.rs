// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO This exist to avoid a cyclic dependency, there has to be another way.

use crate::peer::PeerManager;

use bee_common_pt2::{node::Node, worker::Worker};

use async_trait::async_trait;

use std::convert::Infallible;

pub(crate) struct PeerManagerResWorker {}

#[async_trait]
impl<N: Node> Worker<N> for PeerManagerResWorker {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.register_resource(PeerManager::new());

        Ok(Self {})
    }
}
