// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::peer_state_map::{PeerStateMap, PeerStateMapStats};
use crate::task::Repeat;

use futures::StreamExt;

use std::time::Duration;

pub(crate) const STATE_CHECK_INITIAL: Duration = Duration::from_secs(15);
pub(crate) const STATE_CHECK_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub(crate) struct StateCheckContext {
    pub(crate) peer_state_map: PeerStateMap,
}

pub(crate) fn check_peer_states_fn() -> Repeat<StateCheckContext> {
    Box::new(check_peer_states)
}

fn check_peer_states(ctx: &StateCheckContext) {
    let StateCheckContext { peer_state_map } = ctx;

    log_stats(peer_state_map);
}

fn log_stats(peer_state_map: &PeerStateMap) {
    let PeerStateMapStats {
        num_known,
        num_known_and_connected,
        max_unknown,
        num_unknown_and_connected,
        num_discovered_and_connected,
        max_discovered,
        num_identify_expired,
        num_ping_expired,
        ..
    } = peer_state_map.gen_stats();

    log::info!(
        "Connected peers: known {num_known_and_connected}/{num_known} unknown {num_unknown_and_connected}/{max_unknown} discovered {num_discovered_and_connected}/{max_discovered}.",
    );
    log::debug!("Identify expired #{num_identify_expired} ping expired: #{num_ping_expired}.",)
}

/// `bee-runtime` node worker integration.
pub mod workers {
    use super::*;
    use crate::{manager::workers::GossipManager, server::workers::GossipServer};
    use async_trait::async_trait;
    use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
    use std::{any::TypeId, convert::Infallible};
    use tokio::time::Instant;
    use tokio_stream::wrappers::IntervalStream;

    /// Corresponding worker config.
    pub struct PeerStateCheckerConfig {
        pub(crate) peer_state_map: PeerStateMap,
    }

    /// A node worker, that checks on the connection state of peers.
    ///
    /// NOTE: This type is only exported to be used as a worker dependency.
    #[derive(Default)]
    pub struct PeerStateChecker {}

    #[async_trait]
    impl<N: Node> Worker<N> for PeerStateChecker {
        type Config = PeerStateCheckerConfig;
        type Error = Infallible;

        fn dependencies() -> &'static [TypeId] {
            vec![TypeId::of::<GossipManager>(), TypeId::of::<GossipServer>()].leak()
        }

        async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
            node.spawn::<Self, _, _>(|shutdown_rx| async move {
                log::debug!("Peer state checker running.");

                let PeerStateCheckerConfig { peer_state_map } = &config;

                let interval = tokio::time::interval_at(Instant::now() + STATE_CHECK_INITIAL, STATE_CHECK_INTERVAL);
                let stream = IntervalStream::new(interval);
                let mut ticks = ShutdownStream::new(shutdown_rx, stream);

                while ticks.next().await.is_some() {
                    log_stats(peer_state_map);
                }

                log::debug!("Peer state checker stopped.");
            });

            log::debug!("Peer state checker started.");

            Ok(Self::default())
        }
    }
}
