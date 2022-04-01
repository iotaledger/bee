// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::{swarm::DialError, Multiaddr, PeerId};

use crate::alias;

#[derive(Debug, thiserror::Error)]
// TODO
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("dialing address {0} was denied")]
    DialingAddressDenied(Multiaddr),

    #[error("dialing address {0} failed with cause: {1:?}")]
    DialingAddressFailed(Multiaddr, DialError),

    #[error("dialing peer {} was denied", alias!(.0))]
    DialingPeerDenied(PeerId),

    #[error("dialing peer {} failed with cause: {1:?}", alias!(.0))]
    DialingPeerFailed(PeerId, DialError),
}
