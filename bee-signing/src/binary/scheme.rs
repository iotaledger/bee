// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::binary::ed25519::Seed;

use zeroize::Zeroize;

/// A binary private key.
pub trait PrivateKey: Zeroize + Drop {
    /// Generated private keys type.
    type PrivateKey: PrivateKey;
    /// Matching public key type.
    type PublicKey: PublicKey;
    /// Generated signatures type.
    type Signature: Signature;
    /// Errors occuring while handling private keys.
    type Error;

    /// Deterministically generates and returns a private key from a seed and an index.
    ///
    /// # Arguments
    ///
    /// * `seed`    A seed to deterministically derive a private key from.
    fn generate_from_seed(&self, seed: &Seed) -> Result<Self::PrivateKey, Self::Error>;

    /// Returns the public counterpart of a private key.
    fn generate_public_key(&self) -> Result<Self::PublicKey, Self::Error>;

    /// Generates and returns a signature for a given message.
    ///
    /// # Arguments
    ///
    /// * `message` A slice that holds a message to be signed.
    fn sign(&mut self, message: &[u8]) -> Result<Self::Signature, Self::Error>;
}

/// A binary public key.
pub trait PublicKey {
    /// Matching signature type.
    type Signature: Signature;
    /// Errors occuring while handling public keys.
    type Error;

    /// Verifies a signature for a given message.
    ///
    /// # Arguments
    ///
    /// * `message`     A slice that holds a message to verify a signature for.
    /// * `signature`   The signature to verify.
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> Result<bool, Self::Error>;

    /// Returns the size of the public key.
    fn size(&self) -> usize;
}

/// A binary signature.
pub trait Signature {
    /// Errors occuring while handling public keys.
    type Error;

    /// Returns the size of the signature.
    fn size(&self) -> usize;
}
