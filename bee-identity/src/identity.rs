// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Holds the `Identity` type.

use std::{fmt, path::Path};

use libp2p_core::{identity::PublicKey, PeerId};

use crate::{migration, pem_file, Keypair as Ed25519Keypair, PublicKey as Ed25519PublicKey};

/// Represents a node identity.
#[derive(Clone, Debug)]
pub struct Identity {
    keypair: Ed25519Keypair,
    restored: bool,
}

impl Identity {
    /// Generates a new random identity.
    pub fn generate() -> Self {
        let keypair = Ed25519Keypair::generate();

        Self {
            keypair,
            restored: false,
        }
    }

    /// Restores an [`Identity`] from a [`Keypair`](crate::ed25519::Keypair).
    pub fn from_keypair(keypair: Ed25519Keypair) -> Self {
        Self {
            keypair,
            restored: true,
        }
    }

    /// Returns the backing [`Keypair`](crate::ed25519::Keypair) of this identity.
    pub fn keypair(&self) -> &Ed25519Keypair {
        &self.keypair
    }

    /// Returns the [`PublicKey`](crate::ed25519::PublicKey) of this identity.
    pub fn public_key(&self) -> Ed25519PublicKey {
        self.keypair.public()
    }

    /// Returns the corresponding [`libp2p_core::PeerId`].
    pub fn peer_id(&self) -> PeerId {
        PeerId::from_public_key(&PublicKey::Ed25519(self.keypair.public()))
    }

    /// Returns whether this identity has been restored from a file.
    pub fn is_restored(&self) -> bool {
        self.restored
    }

    /// Stores this identity to a file.
    pub fn store(&self, identity_path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        pem_file::write_keypair_to_pem_file(identity_path, &self.keypair).map_err(|e| {
            log::error!("Failed to write PEM file: {}", e);
            e
        })?;

        Ok(())
    }

    /// Tries to restore an identity from a PEM file. If that doesn't exist or can't be restored generates a new
    /// identity.
    pub fn restore_or_new(
        identity_path: impl AsRef<Path>,
        deprecated_identity_field: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match pem_file::read_keypair_from_pem_file(&identity_path) {
            Ok(keypair) => {
                if deprecated_identity_field.is_some() {
                    log::warn!(
                        "The config file contains an `identity` field which will be ignored. You may safely delete this field to suppress this warning."
                    );
                }
                Ok(Identity::from_keypair(keypair))
            }
            Err(pem_file::PemFileError::Read(_)) => {
                // If we can't read from the file (which means it probably doesn't exist) we either migrate from the
                // existing config or generate a new identity.
                let identity = if let Some(identity_encoded) = deprecated_identity_field {
                    log::warn!(
                        "There is no identity file at `{}`. Migrating identity from the existing config file.",
                        identity_path.as_ref().display(),
                    );

                    let keypair = migration::migrate_keypair(identity_encoded).map_err(|e| {
                        log::error!("Failed to migrate keypair: {}", e);
                        e
                    })?;

                    Identity::from_keypair(keypair)
                } else {
                    log::info!(
                        "There is no identity file at `{}`. Generating a new one.",
                        identity_path.as_ref().display()
                    );

                    Identity::generate()
                };

                identity.store(identity_path)?;

                Ok(identity)
            }
            Err(e) => {
                log::error!("Could not extract private key from PEM file: {}", e);
                Err(Box::new(e))
            }
        }
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded = hex::encode(self.keypair.encode());
        write!(f, "{}", encoded)
    }
}
