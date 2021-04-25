// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::alias;

use libp2p_core::{Multiaddr, PeerId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Dialing address {} was denied.", .0)]
    DialingAddressDenied(Multiaddr),

    #[error("Dialing address {} failed.", .0)]
    DialingAddressFailed(Multiaddr),

    #[error("Dialing peer {} was denied.", alias!(.0))]
    DialingPeerDenied(PeerId),

    #[error("Dialing peer {} failed.", alias!(.0))]
    DialingPeerFailed(PeerId),
}
