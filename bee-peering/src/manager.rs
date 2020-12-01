// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use bee_network::Network;

#[async_trait]
pub trait PeerManager {
    async fn run(self, network: &Network);
}
