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

use crate::ternary::seed::Seed;

use bee_ternary::{T1B1Buf, TritBuf, Trits, T1B1};

use zeroize::Zeroize;

/// Generates a ternary private key.
pub trait PrivateKeyGenerator {
    /// Generated private keys type.
    type PrivateKey: PrivateKey;
    /// Errors occuring while generating private keys.
    type Error;

    /// Deterministically generates and returns a private key from a seed and an index.
    ///
    /// # Arguments
    ///
    /// * `seed`    A seed to deterministically derive a private key from.
    /// * `index`   An index to deterministically derive a private key from.
    ///
    /// # Example
    ///
    /// ```
    /// use bee_crypto::ternary::sponge::Kerl;
    /// use bee_signing::ternary::{
    ///     seed::Seed,
    ///     wots::{WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder},
    ///     PrivateKeyGenerator,
    /// };
    ///
    /// let seed = Seed::from_str("AVXX9XWUSUVKUTWXKTBG9BJVBTZSAISBILKJNVWUHOQNYDMQWXNUCLTTOZGTTLLIYDXXJJGJSEOKVOSSZ").unwrap();
    /// let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
    ///     .security_level(WotsSecurityLevel::Medium)
    ///     .build()
    ///     .unwrap();
    /// let private_key = private_key_generator.generate_from_seed(&seed, 0).unwrap();
    /// ```
    fn generate_from_seed(&self, seed: &Seed, index: u64) -> Result<Self::PrivateKey, Self::Error> {
        self.generate_from_entropy(seed.subseed(index).as_trits())
    }

    /// Deterministically generates and returns a private key from ternary entropy.
    ///
    /// # Arguments
    ///
    /// * `entropy` Entropy to deterministically derive a private key from.
    ///
    /// # Example
    ///
    /// ```
    /// use bee_crypto::ternary::sponge::Kerl;
    /// use bee_signing::ternary::{
    ///     seed::Seed,
    ///     wots::{WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder},
    ///     PrivateKeyGenerator,
    /// };
    ///
    /// let seed = Seed::from_str("AVXX9XWUSUVKUTWXKTBG9BJVBTZSAISBILKJNVWUHOQNYDMQWXNUCLTTOZGTTLLIYDXXJJGJSEOKVOSSZ").unwrap();
    /// let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
    ///     .security_level(WotsSecurityLevel::Medium)
    ///     .build()
    ///     .unwrap();
    /// let private_key = private_key_generator.generate_from_entropy(seed.as_trits()).unwrap();
    /// ```
    fn generate_from_entropy(&self, entropy: &Trits<T1B1>) -> Result<Self::PrivateKey, Self::Error>;
}

/// A ternary private key.
pub trait PrivateKey: Zeroize + Drop {
    /// Matching public key type.
    type PublicKey: PublicKey;
    /// Generated signatures type.
    type Signature: Signature;
    /// Errors occuring while handling private keys.
    type Error;

    /// Returns the public counterpart of a private key.
    ///
    /// # Example
    ///
    /// ```
    /// # use bee_crypto::ternary::sponge::Kerl;
    /// # use bee_signing::ternary::{
    ///     seed::Seed,
    ///     wots::{WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder},
    ///     PrivateKeyGenerator
    /// };
    /// use bee_signing::ternary::PrivateKey;
    ///
    /// # let seed = Seed::from_str("AVXX9XWUSUVKUTWXKTBG9BJVBTZSAISBILKJNVWUHOQNYDMQWXNUCLTTOZGTTLLIYDXXJJGJSEOKVOSSZ").unwrap();
    /// # let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
    ///     .security_level(WotsSecurityLevel::Medium)
    ///     .build()
    ///     .unwrap();
    /// # let private_key = private_key_generator.generate_from_seed(&seed, 0).unwrap();
    /// let public_key = private_key.generate_public_key().unwrap();
    /// ```
    fn generate_public_key(&self) -> Result<Self::PublicKey, Self::Error>;

    /// Generates and returns a signature for a given message.
    ///
    /// # Arguments
    ///
    /// * `message` A slice that holds a message to be signed.
    ///
    /// # Example
    ///
    /// ```
    /// # use bee_crypto::ternary::sponge::Kerl;
    /// # use bee_signing::ternary::{
    ///     seed::Seed,
    ///     wots::{WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder},
    ///     PrivateKeyGenerator
    /// };
    /// use bee_signing::ternary::PrivateKey;
    /// use bee_ternary::{
    ///     T1B1Buf,
    ///     TryteBuf,
    /// };
    ///
    /// # let seed = Seed::from_str("AVXX9XWUSUVKUTWXKTBG9BJVBTZSAISBILKJNVWUHOQNYDMQWXNUCLTTOZGTTLLIYDXXJJGJSEOKVOSSZ").unwrap();
    /// # let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
    ///     .security_level(WotsSecurityLevel::Medium)
    ///     .build()
    ///     .unwrap();
    /// # let mut private_key = private_key_generator.generate_from_seed(&seed, 0).unwrap();
    /// let message = "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD";
    /// let message_trits = TryteBuf::try_from_str(message).unwrap().as_trits().encode::<T1B1Buf>();
    /// let signature = private_key.sign(&message_trits).unwrap();
    /// ```
    fn sign(&mut self, message: &Trits<T1B1>) -> Result<Self::Signature, Self::Error>;
}

/// A ternary public key.
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
    ///
    /// # Example
    ///
    /// ```
    /// # use bee_crypto::ternary::sponge::Kerl;
    /// # use bee_signing::ternary::{
    ///     seed::Seed,
    ///     wots::{WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder},
    ///     PrivateKeyGenerator
    /// };
    /// # use bee_signing::ternary::PrivateKey;
    /// use bee_signing::ternary::PublicKey;
    /// # use bee_ternary::{
    ///     T1B1Buf,
    ///     TryteBuf,
    /// };
    ///
    /// # let seed = Seed::from_str("AVXX9XWUSUVKUTWXKTBG9BJVBTZSAISBILKJNVWUHOQNYDMQWXNUCLTTOZGTTLLIYDXXJJGJSEOKVOSSZ").unwrap();
    /// # let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
    ///     .security_level(WotsSecurityLevel::Medium)
    ///     .build()
    ///     .unwrap();
    /// # let mut private_key = private_key_generator.generate_from_seed(&seed, 0).unwrap();
    /// # let message = "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD";
    /// # let message_trits = TryteBuf::try_from_str(message).unwrap().as_trits().encode::<T1B1Buf>();
    /// # let public_key = private_key.generate_public_key().unwrap();
    /// # let signature = private_key.sign(&message_trits).unwrap();
    /// let valid = public_key.verify(&message_trits, &signature).unwrap();
    /// ```
    fn verify(&self, message: &Trits<T1B1>, signature: &Self::Signature) -> Result<bool, Self::Error>;

    /// Returns the size of the public key.
    fn size(&self) -> usize;

    /// Creates a public key from trits.
    fn from_trits(buf: TritBuf<T1B1Buf>) -> Result<Self, Self::Error>
    where
        Self: std::marker::Sized;

    /// Interprets the public key as trits.
    fn as_trits(&self) -> &Trits<T1B1>;
}

/// A ternary signature.
pub trait Signature {
    /// Errors occuring while handling public keys.
    type Error;

    /// Returns the size of the signature.
    fn size(&self) -> usize;

    /// Creates a signature from trits.
    fn from_trits(buf: TritBuf<T1B1Buf>) -> Result<Self, Self::Error>
    where
        Self: std::marker::Sized;

    /// Interprets the signature as trits.
    fn as_trits(&self) -> &Trits<T1B1>;
}

/// A ternary signature from which a public key can be recovered.
pub trait RecoverableSignature: Signature {
    /// Matching public key type.
    type PublicKey: PublicKey;
    /// Errors occuring while handling recoverable signatures.
    type Error;

    /// Recovers a public key from a signature.
    ///
    /// # Arguments
    ///
    /// * `message` A slice that holds a message to recover the public key from.
    ///
    /// # Example
    ///
    /// ```
    /// # use bee_crypto::ternary::sponge::Kerl;
    /// # use bee_signing::ternary::{
    ///     seed::Seed,
    ///     wots::{WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder},
    ///     PrivateKeyGenerator
    /// };
    /// # use bee_signing::ternary::PrivateKey;
    /// use bee_signing::ternary::RecoverableSignature;
    /// # use bee_ternary::{
    ///     T1B1Buf,
    ///     TryteBuf,
    /// };
    ///
    /// # let seed = Seed::from_str("AVXX9XWUSUVKUTWXKTBG9BJVBTZSAISBILKJNVWUHOQNYDMQWXNUCLTTOZGTTLLIYDXXJJGJSEOKVOSSZ").unwrap();
    /// # let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
    ///     .security_level(WotsSecurityLevel::Medium)
    ///     .build()
    ///     .unwrap();
    /// # let mut private_key = private_key_generator.generate_from_seed(&seed, 0).unwrap();
    /// # let message = "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD";
    /// # let message_trits = TryteBuf::try_from_str(message).unwrap().as_trits().encode::<T1B1Buf>();
    /// # let signature = private_key.sign(&message_trits).unwrap();
    /// let public_key = signature.recover_public_key(&message_trits).unwrap();
    /// ```
    fn recover_public_key(
        &self,
        message: &Trits<T1B1>,
    ) -> Result<Self::PublicKey, <Self as RecoverableSignature>::Error>;
}
