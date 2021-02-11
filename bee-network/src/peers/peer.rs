use libp2p::Multiaddr;

/// Additional information about a peer.
#[derive(Clone, Debug)]
pub struct PeerInfo {
    /// The peer's address.
    pub address: Multiaddr,
    /// The peer's alias.
    pub alias: String,
    /// The type of relation we have with this peer.
    pub relation: PeerRelation,
}

/// Describes the relation with a peer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeerRelation {
    /// Represents a persistent peer. If the connection to such a peer drops, the network will try to reconnect.
    Known,
    /// Represents an ephemeral peer. If the connection to such a peer drops, the network won't try to reconnect.
    Unknown,
}

impl PeerRelation {
    /// Returns whether the peer is known.
    pub fn is_known(&self) -> bool {
        matches!(*self, PeerRelation::Known)
    }

    /// Returns whether the peer is unknown.
    pub fn is_unknown(&self) -> bool {
        matches!(*self, PeerRelation::Unknown)
    }

    /// Upgrades the peer relations.
    pub fn upgrade(&mut self) {
        match self {
            Self::Unknown => *self = Self::Known,
            _ => (),
        }
    }

    /// Downgrades the peer relation.
    pub fn downgrade(&mut self) {
        match self {
            Self::Known => *self = Self::Unknown,
            _ => (),
        }
    }
}
#[derive(Clone)]
pub enum PeerState {
    Disconnected,
    Connected,
}

impl PeerState {
    pub fn is_connected(&self) -> bool {
        matches!(*self, PeerState::Connected)
    }

    pub fn is_disconnected(&self) -> bool {
        matches!(*self, PeerState::Disconnected)
    }
}
