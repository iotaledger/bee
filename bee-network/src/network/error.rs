// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::alias;

use libp2p::{swarm::DialError, Multiaddr, PeerId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Dialing address {0} was denied.")]
    DialingAddressDenied(Multiaddr),

    #[error("Dialing address {0} failed. Cause: {1:?}")]
    DialingAddressFailed(Multiaddr, DialError),

    #[error("Dialing peer {} was denied.", alias!(.0))]
    DialingPeerDenied(PeerId),

    #[error("Dialing peer {} failed. Cause: {1:?}", alias!(.0))]
    DialingPeerFailed(PeerId, DialError),
}
