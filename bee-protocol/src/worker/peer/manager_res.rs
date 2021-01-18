// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO This exist to avoid a cyclic dependency, there has to be another way.

use crate::peer::PeerManager;

use bee_runtime::{node::Node, worker::Worker};

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

    async fn stop(self, node: &mut N) -> Result<(), Self::Error> {
        if let Some(peer_manager) = node.remove_resource::<PeerManager>() {
            for (_, (_, _, shutdown)) in peer_manager.peers.into_inner() {
                // TODO: Should we handle this error?
                let _ = shutdown.send(());
            }
        }

        Ok(())
    }
}
