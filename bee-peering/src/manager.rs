// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_network::Network;

use async_trait::async_trait;

#[async_trait]
pub trait PeerManager {
    type Config;

    async fn start(config: Self::Config, network: &Network) -> Self;
    async fn run(self, network: &Network);
}
