// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::core::connection::ConnectionId;

use std::fmt;

/// Meta information about an established connection.
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
    /// The connection is inbound (local=server).
    Inbound,
    /// The connection is outbound (local=client).
    Outbound,
}

impl Origin {
    /// Returns whether the connection is inbound.
    pub fn is_inbound(&self) -> bool {
        matches!(self, Self::Inbound)
    }

    /// Returns whether the connection is outbound.
    pub fn is_outbound(&self) -> bool {
        matches!(self, Self::Outbound)
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Origin::Inbound => f.write_str("inbound"),
            Origin::Outbound => f.write_str("outbound"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_api() {
        let mut origin = Origin::Inbound;
        assert!(origin.is_inbound());

        origin = Origin::Outbound;
        assert!(origin.is_outbound());
    }

    #[test]
    fn display() {
        assert_eq!(&Origin::Inbound.to_string(), "inbound");
        assert_eq!(&Origin::Outbound.to_string(), "outbound");
    }
}
