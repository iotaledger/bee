// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::alias;

use libp2p::{swarm::DialError, Multiaddr, PeerId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Dialing address {} was denied.", .0)]
    DialingAddressDenied(Multiaddr),

    #[error("Dialing address {} failed. Cause: {:?}", .0, .1)]
    DialingAddressFailed(Multiaddr, DialError),

    #[error("Dialing peer {} was denied.", alias!(.0))]
    DialingPeerDenied(PeerId),

    #[error("Dialing peer {} failed. Cause: {:?}", alias!(.0), .1)]
    DialingPeerFailed(PeerId, DialError),
}
