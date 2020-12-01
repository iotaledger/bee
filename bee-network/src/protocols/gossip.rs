// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::NETWORK_ID;

use futures::prelude::*;
use libp2p::core::{
    muxing::{StreamMuxerBox, SubstreamRef},
    InboundUpgrade, Negotiated, OutboundUpgrade, UpgradeInfo,
};

use std::{
    iter,
    sync::{atomic::Ordering, Arc},
};

pub type GossipSubstream = Negotiated<SubstreamRef<Arc<StreamMuxerBox>>>;

#[derive(Default, Debug, Clone)]
pub struct GossipProtocol;

impl UpgradeInfo for GossipProtocol {
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

impl InboundUpgrade<GossipSubstream> for GossipProtocol {
    type Output = GossipSubstream;
    type Error = ();
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, stream: GossipSubstream, _: Self::Info) -> Self::Future {
        future::ok(stream)
    }
}

impl OutboundUpgrade<GossipSubstream> for GossipProtocol {
    type Output = GossipSubstream;
    type Error = ();
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, stream: GossipSubstream, _: Self::Info) -> Self::Future {
        future::ok(stream)
    }
}
