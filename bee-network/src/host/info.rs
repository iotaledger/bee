// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::core::connection::ConnectionId;

use std::fmt;

#[derive(Clone, Debug)]
pub struct ConnectionInfo {
    /// The assigned connection id.
    pub id: ConnectionId,
    /// Whether the connection is inbound or outbound.
    pub origin: Origin,
}

/// Describes direction of an established connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Origin {
    /// The connection is inbound (server).
    Inbound,
    /// The connection is outbound (client).
    Outbound,
}

impl Origin {
    /// Returns whether the connection is inbound.
    pub fn is_inbound(&self) -> bool {
        self.eq(&Self::Inbound)
    }

    /// Returns whether the connection is outbound.
    pub fn is_outbound(&self) -> bool {
        self.eq(&Self::Outbound)
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            Origin::Outbound => "outbound",
            Origin::Inbound => "inbound",
        };
        write!(f, "{}", s)
    }
}
