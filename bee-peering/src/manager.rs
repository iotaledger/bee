// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use bee_network::NetworkController;

#[async_trait]
pub trait PeerManager {
    type Config;

    async fn new(config: Self::Config, network: &NetworkController) -> Self;
    async fn run(self, network: &NetworkController);
}
