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

//! Winternitz One Time Signature scheme.
//! https://eprint.iacr.org/2011/191.pdf.

mod normalize;
mod shake;
mod sponge;

pub use normalize::{normalize, Error as NormalizeError};
pub use shake::{WotsShakePrivateKeyGenerator, WotsShakePrivateKeyGeneratorBuilder};
pub use sponge::{WotsSpongePrivateKeyGenerator, WotsSpongePrivateKeyGeneratorBuilder};

use crate::ternary::{PrivateKey, PublicKey, RecoverableSignature, Signature, SIGNATURE_FRAGMENT_LENGTH};

use bee_common_derive::{SecretDebug, SecretDisplay, SecretDrop};
use bee_crypto::ternary::{sponge::Sponge, HASH_LENGTH};
use bee_ternary::{T1B1Buf, TritBuf, Trits, Tryte, T1B1};

use thiserror::Error;
use zeroize::Zeroize;

use std::{
    convert::TryFrom,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
};

/// Errors occuring during WOTS operations.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// Missing security level in generator.
    #[error("Missing security level in generator.")]
    MissingSecurityLevel,
    /// Failed sponge operation.
    #[error("Failed sponge operation.")]
    FailedSpongeOperation,
    /// Invalid entropy length.
    #[error("Invalid entropy length, should be 243 trits.")]
    InvalidEntropyLength(usize),
    /// Invalid message length.
    #[error("Invalid message length, should be 243 trits.")]
    InvalidMessageLength(usize),
    /// Invalid public key length.
    #[error("Invalid public key length, should be 243 trits.")]
    InvalidPublicKeyLength(usize),
    /// Invalid signature length.
    #[error("Invalid signature length, should be 243 trits.")]
    InvalidSignatureLength(usize),
    /// Last trit of the entropy is not null.
    #[error("Last trit of the entropy is not null.")]
    NonNullEntropyLastTrit,
}

/// Available WOTS security levels.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum WotsSecurityLevel {
    /// Low security.
    Low = 1,
    /// Medium security.
    Medium = 2,
    /// High security.
    High = 3,
}

impl Default for WotsSecurityLevel {
    fn default() -> Self {
        WotsSecurityLevel::Medium
    }
}

/// Winternitz One Time Signature private key.
#[derive(SecretDebug, SecretDisplay, SecretDrop)]
pub struct WotsPrivateKey<S> {
    pub(crate) state: TritBuf<T1B1Buf>,
    pub(crate) sponge: PhantomData<S>,
}

impl<S> Zeroize for WotsPrivateKey<S> {
    fn zeroize(&mut self) {
        // This unsafe is fine since we only reset the whole buffer with zeros, there is no alignement issues.
        unsafe { self.state.as_i8_slice_mut().zeroize() }
    }
}

impl<S: Sponge + Default> PrivateKey for WotsPrivateKey<S> {
    type PublicKey = WotsPublicKey<S>;
    type Signature = WotsSignature<S>;
    type Error = Error;

    fn generate_public_key(&self) -> Result<Self::PublicKey, Self::Error> {
        let mut sponge = S::default();
        let mut hashed_private_key = self.state.clone();
        let security = self.state.len() / SIGNATURE_FRAGMENT_LENGTH;
        let mut digests = TritBuf::<T1B1Buf>::zeros(security * HASH_LENGTH);
        let mut public_key_state = TritBuf::<T1B1Buf>::zeros(HASH_LENGTH);

        // Hash each chunk of the private key the maximum amount of times.
        for chunk in hashed_private_key.chunks_mut(HASH_LENGTH) {
            for _ in 0..Tryte::MAX_VALUE as i8 - Tryte::MIN_VALUE as i8 {
                sponge.absorb(chunk).map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge
                    .squeeze_into(chunk)
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge.reset();
            }
        }

        // Create one digest per fragment of the private key.
        for (i, chunk) in hashed_private_key.chunks(SIGNATURE_FRAGMENT_LENGTH).enumerate() {
            sponge
                .digest_into(chunk, &mut digests[i * HASH_LENGTH..(i + 1) * HASH_LENGTH])
                .map_err(|_| Self::Error::FailedSpongeOperation)?;
        }

        // Hash the digests together to produce the public key.
        sponge
            .digest_into(&digests, &mut public_key_state)
            .map_err(|_| Self::Error::FailedSpongeOperation)?;

        Ok(Self::PublicKey {
            state: public_key_state,
            sponge: PhantomData,
        })
    }

    fn sign(&mut self, message: &Trits<T1B1>) -> Result<Self::Signature, Self::Error> {
        if message.len() != HASH_LENGTH {
            return Err(Error::InvalidMessageLength(message.len()));
        }

        let mut sponge = S::default();
        let mut signature = self.state.clone();

        for (i, chunk) in signature.chunks_mut(HASH_LENGTH).enumerate() {
            // Safe to unwrap because 3 trits can't underflow/overflow an i8.
            let val = i8::try_from(&message[i * 3..i * 3 + 3]).unwrap();

            // Hash each chunk of the private key an amount of times given by the corresponding byte of the message.
            for _ in 0..(Tryte::MAX_VALUE as i8 - val) {
                sponge.absorb(chunk).map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge
                    .squeeze_into(chunk)
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge.reset();
            }
        }

        Ok(Self::Signature {
            state: signature,
            sponge: PhantomData,
        })
    }
}

impl<S: Sponge + Default> WotsPrivateKey<S> {
    /// Returns the inner trits.
    pub fn as_trits(&self) -> &Trits<T1B1> {
        &self.state
    }
}

/// Winternitz One Time Signature public key.
pub struct WotsPublicKey<S> {
    state: TritBuf<T1B1Buf>,
    sponge: PhantomData<S>,
}

impl<S: Sponge + Default> PublicKey for WotsPublicKey<S> {
    type Signature = WotsSignature<S>;
    type Error = Error;

    fn verify(&self, message: &Trits<T1B1>, signature: &Self::Signature) -> Result<bool, Self::Error> {
        Ok(signature.recover_public_key(message)?.state == self.state)
    }

    fn size(&self) -> usize {
        self.state.len()
    }

    fn from_trits(state: TritBuf<T1B1Buf>) -> Result<Self, Self::Error> {
        if state.len() != HASH_LENGTH {
            return Err(Error::InvalidPublicKeyLength(state.len()));
        }

        Ok(Self {
            state,
            sponge: PhantomData,
        })
    }

    fn as_trits(&self) -> &Trits<T1B1> {
        &self.state
    }
}

impl<S: Sponge + Default> Display for WotsPublicKey<S> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.as_trits().iter_trytes().map(char::from).collect::<String>()
        )
    }
}

/// Winternitz One Time Signature signature.
pub struct WotsSignature<S> {
    state: TritBuf<T1B1Buf>,
    sponge: PhantomData<S>,
}

impl<S: Sponge + Default> Signature for WotsSignature<S> {
    type Error = Error;

    fn size(&self) -> usize {
        self.state.len()
    }

    fn from_trits(state: TritBuf<T1B1Buf>) -> Result<Self, Self::Error> {
        if state.len() % SIGNATURE_FRAGMENT_LENGTH != 0 {
            return Err(Error::InvalidSignatureLength(state.len()));
        }

        Ok(Self {
            state,
            sponge: PhantomData,
        })
    }

    fn as_trits(&self) -> &Trits<T1B1> {
        &self.state
    }
}

impl<S: Sponge + Default> RecoverableSignature for WotsSignature<S> {
    type PublicKey = WotsPublicKey<S>;
    type Error = Error;

    fn recover_public_key(
        &self,
        message: &Trits<T1B1>,
    ) -> Result<Self::PublicKey, <Self as RecoverableSignature>::Error> {
        if message.len() != HASH_LENGTH {
            return Err(Error::InvalidMessageLength(message.len()));
        }

        let mut sponge = S::default();
        let mut public_key_state = TritBuf::<T1B1Buf>::zeros(HASH_LENGTH);
        let security = self.state.len() / SIGNATURE_FRAGMENT_LENGTH;
        let mut digests = TritBuf::<T1B1Buf>::zeros(security * HASH_LENGTH);
        let mut hashed_signature = self.state.clone();

        for (i, chunk) in hashed_signature.chunks_mut(HASH_LENGTH).enumerate() {
            // Safe to unwrap because 3 trits can't underflow/overflow an i8.
            let val = i8::try_from(&message[i * 3..i * 3 + 3]).unwrap();

            // Hash each chunk of the signature an amount of times given by the corresponding byte of the message.
            for _ in 0..(val - Tryte::MIN_VALUE as i8) {
                sponge
                    .absorb(chunk)
                    .map_err(|_| <Self as RecoverableSignature>::Error::FailedSpongeOperation)?;
                sponge
                    .squeeze_into(chunk)
                    .map_err(|_| <Self as RecoverableSignature>::Error::FailedSpongeOperation)?;
                sponge.reset();
            }
        }

        // Create one digest per fragment of the signature.
        for (i, chunk) in hashed_signature.chunks(SIGNATURE_FRAGMENT_LENGTH).enumerate() {
            sponge
                .digest_into(chunk, &mut digests[i * HASH_LENGTH..(i + 1) * HASH_LENGTH])
                .map_err(|_| <Self as RecoverableSignature>::Error::FailedSpongeOperation)?;
        }

        // Hash the digests together to recover the public key.
        sponge
            .digest_into(&digests, &mut public_key_state)
            .map_err(|_| <Self as RecoverableSignature>::Error::FailedSpongeOperation)?;

        Ok(Self::PublicKey {
            state: public_key_state,
            sponge: PhantomData,
        })
    }
}

impl<S: Sponge + Default> Display for WotsSignature<S> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.as_trits().iter_trytes().map(char::from).collect::<String>()
        )
    }
}
