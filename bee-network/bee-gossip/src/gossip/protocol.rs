// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use futures::future;
use libp2p::core::{
    upgrade::{InboundUpgrade, OutboundUpgrade},
    ProtocolName, UpgradeInfo,
};

use core::fmt;
use std::{io, iter};

#[derive(Clone, Debug)]
pub struct GossipProtocolName(pub(crate) &'static str);

impl GossipProtocolName {
    fn as_str(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for GossipProtocolName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl ProtocolName for GossipProtocolName {
    fn protocol_name(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct GossipProtocol {
    name: GossipProtocolName,
}

impl GossipProtocol {
    pub fn name(&self) -> &GossipProtocolName {
        &self.name
    }
}

impl GossipProtocol {
    pub(crate) fn new(name: &'static str) -> Self {
        Self {
            name: GossipProtocolName(name),
        }
    }
}

impl UpgradeInfo for GossipProtocol {
    type Info = &'static str;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        log::trace!("Requested protocol info: {}", self.name);

        iter::once(self.name().as_str())
    }
}

impl<S> InboundUpgrade<S> for GossipProtocol {
    type Output = S;
    type Error = io::Error;
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, substream: Self::Output, _: Self::Info) -> Self::Future {
        log::trace!("inbound upgrade successful");

        future::ok(substream)
    }
}

impl<S> OutboundUpgrade<S> for GossipProtocol {
    type Output = S;
    type Error = io::Error;
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, substream: Self::Output, _: Self::Info) -> Self::Future {
        log::trace!("outbound upgrade successful");

        future::ok(substream)
    }
}
