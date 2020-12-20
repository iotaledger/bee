// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod topics;

use bee_common_pt2::{node::Node, worker::Worker};

use async_trait::async_trait;
use log::info;

use std::convert::Infallible;

#[derive(Default)]
pub struct Mqtt;

#[async_trait]
impl<N: Node> Worker<N> for Mqtt {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|_shutdown| async move {
            info!("Running.");

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
