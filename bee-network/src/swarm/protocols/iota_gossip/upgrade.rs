// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use super::id::IotaGossipIdentifier;

use futures::{future, AsyncRead, AsyncWrite};
use libp2p::{core::UpgradeInfo, InboundUpgrade, OutboundUpgrade};
use log::*;

use std::{io, iter};

#[derive(Debug, Clone)]
pub struct IotaGossipProtocolUpgrade {
    id: IotaGossipIdentifier,
}

impl IotaGossipProtocolUpgrade {
    pub fn new(id: IotaGossipIdentifier) -> Self {
        Self { id }
    }
}

impl UpgradeInfo for IotaGossipProtocolUpgrade {
    type Info = IotaGossipIdentifier;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        trace!("gossip upgrade: protocol info query: {}", self.id);

        iter::once(self.id.clone())
    }
}

impl<S> InboundUpgrade<S> for IotaGossipProtocolUpgrade
where
    S: AsyncWrite + AsyncWrite + Unpin + Send,
{
    type Output = S;
    type Error = io::Error;
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, stream: S, info: Self::Info) -> Self::Future {
        debug!("gossip upgrade: inbound: {}", info);

        future::ok(stream)
    }
}

impl<S> OutboundUpgrade<S> for IotaGossipProtocolUpgrade
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    type Output = S;
    type Error = io::Error;
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, stream: S, info: Self::Info) -> Self::Future {
        debug!("gossip upgrade: outbound: {}", info);

        future::ok(stream)
    }
}
