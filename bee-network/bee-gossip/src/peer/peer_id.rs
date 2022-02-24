// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Ed25519PublicKey;
use libp2p_core::multihash::Multihash;

use std::{fmt, ops, str::FromStr};

const DISPLAY_LENGTH: usize = 16;

/// A wrapper around a `libp2p_core::PeerId` with a custom `Display` implementation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PeerId(libp2p_core::PeerId);

impl PeerId {
    /// Tries to turn a `Multihash` into a `PeerId`.
    ///
    /// If the multihash does not use a valid hashing algorithm for peer IDs,
    /// or the hash value does not satisfy the constraints for a hashed
    /// peer ID, it is returned as an `Err`.
    pub fn from_multihash(multihash: Multihash) -> Result<Self, Multihash> {
        libp2p_core::PeerId::from_multihash(multihash).map(|a| a.into())
    }

    /// Builds a `PeerId` from a public key.
    pub fn from_public_key(public_key: Ed25519PublicKey) -> Self {
        libp2p_core::PeerId::from_public_key(&libp2p_core::identity::PublicKey::Ed25519(public_key)).into()
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.to_base58()[..DISPLAY_LENGTH].fmt(f)
    }
}

impl ops::Deref for PeerId {
    type Target = libp2p_core::PeerId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<libp2p_core::PeerId> for PeerId {
    fn from(peer_id: libp2p_core::PeerId) -> Self {
        Self(peer_id)
    }
}

impl FromStr for PeerId {
    type Err = <libp2p_core::PeerId as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        libp2p_core::PeerId::from_str(s).map(|p| p.into())
    }
}
