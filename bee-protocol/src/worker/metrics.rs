// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::ProtocolMetrics;

use bee_common::{node::Node, worker::Worker};

use async_trait::async_trait;

use std::convert::Infallible;

pub(crate) struct MetricsWorker {}

#[async_trait]
impl<N: Node> Worker<N> for MetricsWorker {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.register_resource(ProtocolMetrics::new());

        Ok(Self {})
    }
}
