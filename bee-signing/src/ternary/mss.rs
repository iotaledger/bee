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

//! Merkle Signature Scheme.

use crate::ternary::{
    seed::Seed, PrivateKey, PrivateKeyGenerator, PublicKey, RecoverableSignature, Signature, SIGNATURE_FRAGMENT_LENGTH,
};

use bee_common_derive::{SecretDebug, SecretDisplay, SecretDrop};
use bee_crypto::ternary::{sponge::Sponge, HASH_LENGTH};
use bee_ternary::{T1B1Buf, TritBuf, Trits, T1B1};

use thiserror::Error;
use zeroize::Zeroize;

use std::marker::PhantomData;

const MAX_MSS_DEPTH: u8 = 20;

/// Errors occuring during MSS operations.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// Invalid MSS depth provided.
    #[error("Invalid MSS depth provided.")]
    InvalidDepth(u8),
    /// Missing MSS depth.
    #[error("Missing MSS depth.")]
    MissingDepth,
    /// Missing signature index.
    #[error("Missing signature index.")]
    MissingSignatureIndex,
    /// Missing underlying private key generator.
    #[error("Missing underlying private key generator.")]
    MissingGenerator,
    /// Underlying private key generation failed.
    #[error("Underlying private key generation failed.")]
    FailedUnderlyingPrivateKeyGeneration,
    /// Underlying public key generation failed.
    #[error("Underlying public key generation failed.")]
    FailedUnderlyingPublicKeyGeneration,
    /// Underlying signature generation failed.
    #[error("Underlying signature generation failed.")]
    FailedUnderlyingSignatureGeneration,
    /// Underlying public key recovery failed.
    #[error("Underlying public key recovery failed.")]
    FailedUnderlyingPublicKeyRecovery,
    /// Public key generation failed.
    #[error("Public key generation failed..")]
    PublicKeyGenerationFailed,
    /// Signature generation failed.
    #[error("Signature generation failed..")]
    SignatureGenerationFailed,
    /// Failed sponge operation.
    #[error("Failed sponge operation.")]
    FailedSpongeOperation,
    /// Seed generation failed.
    #[error("Seed generation failed.")]
    FailedSeedGeneration,
    /// Signatures exhausted.
    #[error("Signatures exhausted.")]
    SignaturesExhausted,
    /// Invalid signature size.
    #[error("Invalid signature size.")]
    InvalidSignatureSize,
}

/// Merkle Signature Scheme private key generator builder.
pub struct MssPrivateKeyGeneratorBuilder<S, G> {
    depth: Option<u8>,
    generator: Option<G>,
    sponge: PhantomData<S>,
}

impl<S, G> Default for MssPrivateKeyGeneratorBuilder<S, G>
where
    S: Sponge + Default,
    G: PrivateKeyGenerator,
{
    fn default() -> Self {
        Self {
            depth: None,
            generator: None,
            sponge: PhantomData,
        }
    }
}

impl<S, G> MssPrivateKeyGeneratorBuilder<S, G>
where
    S: Sponge + Default,
    G: PrivateKeyGenerator,
{
    /// Sets the depth of the MSS.
    pub fn depth(mut self, depth: u8) -> Self {
        self.depth.replace(depth);
        self
    }

    /// Sets the underlying private key generator.
    pub fn generator(mut self, generator: G) -> Self {
        self.generator.replace(generator);
        self
    }

    /// Builds the private key generator.
    pub fn build(self) -> Result<MssPrivateKeyGenerator<S, G>, Error> {
        let depth = match self.depth {
            Some(depth) => match depth {
                0..=MAX_MSS_DEPTH => depth,
                _ => return Err(Error::InvalidDepth(depth)),
            },
            None => return Err(Error::MissingDepth),
        };

        Ok(MssPrivateKeyGenerator {
            depth,
            generator: self.generator.ok_or(Error::MissingGenerator)?,
            sponge: PhantomData,
        })
    }
}

/// Merkle Signature Scheme private key generator.
pub struct MssPrivateKeyGenerator<S, G> {
    depth: u8,
    generator: G,
    sponge: PhantomData<S>,
}

impl<S, G> PrivateKeyGenerator for MssPrivateKeyGenerator<S, G>
where
    S: Sponge + Default,
    G: PrivateKeyGenerator,
    <<<G as PrivateKeyGenerator>::PrivateKey as PrivateKey>::PublicKey as PublicKey>::Signature: RecoverableSignature,
{
    type PrivateKey = MssPrivateKey<S, G::PrivateKey>;
    type Error = Error;

    fn generate_from_entropy(&self, entropy: &Trits<T1B1>) -> Result<Self::PrivateKey, Self::Error> {
        let seed = Seed::from_trits(entropy.to_buf()).map_err(|_| Error::FailedSeedGeneration)?;
        let mut sponge = S::default();
        let mut keys = Vec::with_capacity(1 << (self.depth - 1));
        let mut tree = TritBuf::<T1B1Buf>::zeros(((1 << self.depth) - 1) * HASH_LENGTH);

        // Generate all the underlying private keys and public keys.
        for key_index in 0..(1 << (self.depth - 1)) {
            let underlying_private_key = self
                .generator
                .generate_from_entropy(seed.subseed(key_index).as_trits())
                .map_err(|_| Self::Error::FailedUnderlyingPrivateKeyGeneration)?;
            let underlying_public_key = underlying_private_key
                .generate_public_key()
                .map_err(|_| Self::Error::FailedUnderlyingPublicKeyGeneration)?;
            let tree_index = ((1 << (self.depth - 1)) + key_index - 1) as usize;

            keys.push(underlying_private_key);
            tree[tree_index * HASH_LENGTH..(tree_index + 1) * HASH_LENGTH].copy_from(underlying_public_key.as_trits());
        }

        // Create the merkle tree by hashing from botton to top.
        for depth in (0..self.depth - 1).rev() {
            for i in 0..(1 << depth) {
                let index = (1 << depth) + i - 1;
                let left_index = index * 2 + 1;
                let right_index = left_index + 1;

                sponge
                    .absorb(&tree[left_index * HASH_LENGTH..(left_index + 1) * HASH_LENGTH])
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge
                    .absorb(&tree[right_index * HASH_LENGTH..(right_index + 1) * HASH_LENGTH])
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge
                    .squeeze_into(&mut tree[index * HASH_LENGTH..(index + 1) * HASH_LENGTH])
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge.reset();
            }
        }

        Ok(MssPrivateKey {
            depth: self.depth,
            index: 0,
            keys,
            tree,
            sponge: PhantomData,
        })
    }
}

/// Merkle Signature Scheme private key.
#[derive(SecretDebug, SecretDisplay, SecretDrop)]
pub struct MssPrivateKey<S, K: Zeroize> {
    depth: u8,
    index: u64,
    keys: Vec<K>,
    tree: TritBuf<T1B1Buf>,
    sponge: PhantomData<S>,
}

impl<S, K: Zeroize> Zeroize for MssPrivateKey<S, K> {
    fn zeroize(&mut self) {
        for key in self.keys.iter_mut() {
            key.zeroize();
        }
        // This unsafe is fine since we only reset the whole buffer with zeros, there is no alignement issues.
        unsafe { self.tree.as_i8_slice_mut().zeroize() }
    }
}

impl<S, K> PrivateKey for MssPrivateKey<S, K>
where
    S: Sponge + Default,
    K: PrivateKey,
    <<K as PrivateKey>::PublicKey as PublicKey>::Signature: RecoverableSignature,
{
    type PublicKey = MssPublicKey<S, K::PublicKey>;
    type Signature = MssSignature<S>;
    type Error = Error;

    fn generate_public_key(&self) -> Result<Self::PublicKey, Self::Error> {
        Ok(Self::PublicKey::from_trits(self.tree[0..HASH_LENGTH].to_buf())
            .map_err(|_| Error::PublicKeyGenerationFailed)?
            .depth(self.depth))
    }

    fn sign(&mut self, message: &Trits<T1B1>) -> Result<Self::Signature, Self::Error> {
        if self.index as usize >= self.keys.len() {
            return Err(Error::SignaturesExhausted);
        }

        let underlying_private_key = &mut self.keys[self.index as usize];
        let underlying_signature = underlying_private_key
            .sign(message)
            .map_err(|_| Self::Error::FailedUnderlyingSignatureGeneration)?;
        let mut state = TritBuf::<T1B1Buf>::zeros(underlying_signature.size() + SIGNATURE_FRAGMENT_LENGTH);
        let mut tree_index = ((1 << (self.depth - 1)) + self.index - 1) as usize;
        let mut sibling_index;
        let mut i = 0;

        // Copy the underlying signature.
        state[0..underlying_signature.size()].copy_from(underlying_signature.as_trits());

        // Append the merkle branch to the signature.
        while tree_index != 0 {
            if tree_index % 2 != 0 {
                sibling_index = tree_index + 1;
                tree_index /= 2;
            } else {
                sibling_index = tree_index - 1;
                tree_index = (tree_index - 1) / 2;
            }

            state[underlying_signature.size() + i * HASH_LENGTH..underlying_signature.size() + (i + 1) * HASH_LENGTH]
                .copy_from(&self.tree[sibling_index * HASH_LENGTH..(sibling_index + 1) * HASH_LENGTH]);
            i += 1;
        }

        self.index += 1;

        Ok(Self::Signature::from_trits(state)
            .map_err(|_| Error::SignatureGenerationFailed)?
            .index(self.index - 1))
    }
}

/// Merkle Signature Scheme public key.
pub struct MssPublicKey<S, K> {
    state: TritBuf<T1B1Buf>,
    depth: Option<u8>,
    sponge: PhantomData<S>,
    key: PhantomData<K>,
}

impl<S, K> MssPublicKey<S, K>
where
    S: Sponge + Default,
    K: PublicKey,
{
    /// Sets the depth of the public key.
    pub fn depth(mut self, depth: u8) -> Self {
        self.depth.replace(depth);
        self
    }
}

impl<S, K> PublicKey for MssPublicKey<S, K>
where
    S: Sponge + Default,
    K: PublicKey,
    <K as PublicKey>::Signature: RecoverableSignature,
{
    type Signature = MssSignature<S>;
    type Error = Error;

    fn verify(&self, message: &Trits<T1B1>, signature: &Self::Signature) -> Result<bool, Self::Error> {
        if signature.size() % SIGNATURE_FRAGMENT_LENGTH != 0 || signature.size() < 2 * SIGNATURE_FRAGMENT_LENGTH {
            return Err(Error::InvalidSignatureSize);
        }

        let depth = self.depth.ok_or(Error::MissingDepth)?;
        let signature_index = signature.index.ok_or(Error::MissingSignatureIndex)?;

        let mut sponge = S::default();
        let underlying_signature = K::Signature::from_trits(
            signature.state[0..((signature.state.len() / SIGNATURE_FRAGMENT_LENGTH) - 1) * SIGNATURE_FRAGMENT_LENGTH]
                .to_buf(),
        )
        .map_err(|_| Error::FailedUnderlyingSignatureGeneration)?;

        // Safe to unwrap since we already validated the signature size.
        let siblings: &Trits<T1B1> = signature.state.chunks(SIGNATURE_FRAGMENT_LENGTH).last().unwrap();
        // Recovers underlying public key from underlying signature.
        let underlying_public_key = underlying_signature
            .recover_public_key(message)
            .map_err(|_| Self::Error::FailedUnderlyingPublicKeyRecovery)?;
        let mut hash = TritBuf::<T1B1Buf>::zeros(HASH_LENGTH);

        hash.copy_from(underlying_public_key.as_trits());

        // Hash the underlying public key with the merkle branch to recover the merkle root.
        let mut j = 1;
        for (i, sibling) in siblings.chunks(HASH_LENGTH).enumerate() {
            if depth - 1 == i as u8 {
                break;
            }

            if signature_index & j != 0 {
                sponge.absorb(sibling).map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge.absorb(&hash).map_err(|_| Self::Error::FailedSpongeOperation)?;
            } else {
                sponge.absorb(&hash).map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge
                    .absorb(&sibling)
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
            }
            sponge
                .squeeze_into(&mut hash)
                .map_err(|_| Self::Error::FailedSpongeOperation)?;
            sponge.reset();

            j <<= 1;
        }

        Ok(hash == self.state)
    }

    fn size(&self) -> usize {
        self.state.len()
    }

    fn from_trits(state: TritBuf<T1B1Buf>) -> Result<Self, Self::Error> {
        Ok(Self {
            state,
            depth: None,
            sponge: PhantomData,
            key: PhantomData,
        })
    }

    fn as_trits(&self) -> &Trits<T1B1> {
        &self.state
    }
}

/// Merkle Signature Scheme signature.
pub struct MssSignature<S> {
    state: TritBuf<T1B1Buf>,
    index: Option<u64>,
    sponge: PhantomData<S>,
}

impl<S: Sponge + Default> MssSignature<S> {
    /// Set the index of the signature.
    pub fn index(mut self, index: u64) -> Self {
        self.index.replace(index);
        self
    }
}

impl<S: Sponge + Default> Signature for MssSignature<S> {
    type Error = Error;

    fn size(&self) -> usize {
        self.state.len()
    }

    fn from_trits(state: TritBuf<T1B1Buf>) -> Result<Self, Error> {
        Ok(Self {
            state,
            index: None,
            sponge: PhantomData,
        })
    }

    fn as_trits(&self) -> &Trits<T1B1> {
        &self.state
    }
}
