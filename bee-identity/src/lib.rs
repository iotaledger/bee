// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Creation and persistence of local node identities.

#![deny(missing_docs)]

pub mod identity;
/// Ed25519 keys.
pub mod ed25519 {
    #[doc(inline)]
    pub use libp2p_core::identity::ed25519::{Keypair, PublicKey, SecretKey};
}
pub mod pem_file;

pub use crate::identity::Identity;
pub use crate::ed25519::{Keypair, PublicKey, SecretKey};
#[doc(inline)]
pub use libp2p_core::PeerId;
