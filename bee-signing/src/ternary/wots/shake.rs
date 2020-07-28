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

use crate::ternary::{
    wots::{Error as WotsError, WotsPrivateKey, WotsSecurityLevel},
    PrivateKeyGenerator, SIGNATURE_FRAGMENT_LENGTH,
};

use bee_crypto::ternary::{
    bigint::{binary_representation::U8Repr, endianness::BigEndian, I384, T242, T243},
    sponge::Sponge,
    HASH_LENGTH,
};
use bee_ternary::{Btrit, T1B1Buf, TritBuf, Trits, T1B1};

use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake256,
};

use std::marker::PhantomData;

/// Shake-based Winternitz One Time Signature private key generator builder.
#[derive(Default)]
pub struct WotsShakePrivateKeyGeneratorBuilder<S> {
    security_level: Option<WotsSecurityLevel>,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> WotsShakePrivateKeyGeneratorBuilder<S> {
    /// Sets the security level of the private key.
    pub fn security_level(mut self, security_level: WotsSecurityLevel) -> Self {
        self.security_level.replace(security_level);
        self
    }

    /// Builds the private key generator.
    pub fn build(self) -> Result<WotsShakePrivateKeyGenerator<S>, WotsError> {
        Ok(WotsShakePrivateKeyGenerator {
            security_level: self.security_level.ok_or(WotsError::MissingSecurityLevel)?,
            _sponge: PhantomData,
        })
    }
}

/// Shake-based Winternitz One Time Signature private key generator.
pub struct WotsShakePrivateKeyGenerator<S> {
    security_level: WotsSecurityLevel,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> PrivateKeyGenerator for WotsShakePrivateKeyGenerator<S> {
    type PrivateKey = WotsPrivateKey<S>;
    type Error = WotsError;

    /// Derives a private key from entropy using the SHAKE256 extendable-output function.
    /// The entropy must be a slice of exactly 243 trits where the last trit is zero.
    /// Derives its security assumptions from the properties of the underlying SHAKE function.
    fn generate_from_entropy(&self, entropy: &Trits<T1B1>) -> Result<Self::PrivateKey, Self::Error> {
        if entropy.len() != HASH_LENGTH {
            return Err(WotsError::InvalidEntropyLength(entropy.len()));
        }

        if entropy[HASH_LENGTH - 1] != Btrit::Zero {
            return Err(WotsError::NonNullEntropyLastTrit);
        }

        let mut state = TritBuf::<T1B1Buf>::zeros(self.security_level as usize * SIGNATURE_FRAGMENT_LENGTH);
        let mut shake = Shake256::default();
        let mut ternary_buffer = T243::<Btrit>::default();
        ternary_buffer.copy_from(entropy);
        let mut binary_buffer: I384<BigEndian, U8Repr> = ternary_buffer.into_t242().into();

        shake.update(&binary_buffer[..]);
        let mut reader = shake.finalize_xof();

        for chunk in state.chunks_mut(HASH_LENGTH) {
            reader.read(&mut binary_buffer[..]);
            ternary_buffer = T242::from_i384_ignoring_mst(binary_buffer).into_t243();

            chunk.copy_from(&ternary_buffer);
        }

        Ok(Self::PrivateKey {
            state,
            sponge: PhantomData,
        })
    }
}
