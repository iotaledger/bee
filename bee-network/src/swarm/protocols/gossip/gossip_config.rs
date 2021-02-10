use crate::NETWORK_ID;

use futures::{future, AsyncRead, AsyncWrite};
use libp2p::{core::UpgradeInfo, InboundUpgrade, OutboundUpgrade};
use log::trace;

use std::{iter, sync::atomic::Ordering};

/// Configuration for an upgrade to the `IotaGossip` protocol.
#[derive(Debug, Clone, Default)]
pub struct GossipConfig;

impl UpgradeInfo for GossipConfig {
    type Info = Vec<u8>;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(
            format!("/iota-gossip/{}/1.0.0", NETWORK_ID.load(Ordering::Relaxed))
                .as_bytes()
                .to_vec(),
        )
    }
}

impl<C> InboundUpgrade<C> for GossipConfig
where
    C: AsyncRead + AsyncWrite + Unpin,
{
    type Output = C;
    type Error = ();
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, stream: C, _: Self::Info) -> Self::Future {
        // NOTE: do nothing, just return the stream.
        trace!("Upgrading inbound connection to gossip protocol.");
        future::ok(stream)
    }
}

impl<C> OutboundUpgrade<C> for GossipConfig
where
    C: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Output = C;
    type Error = ();
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, stream: C, _: Self::Info) -> Self::Future {
        // NOTE: do nothing, just return the stream.
        trace!("Upgrading outbound connection to gossip protocol.");
        future::ok(stream)
    }
}
