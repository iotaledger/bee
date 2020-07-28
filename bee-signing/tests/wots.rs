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

use bee_crypto::ternary::sponge::Kerl;
use bee_signing::ternary::{
    wots::{Error as WotsError, WotsPublicKey, WotsSecurityLevel, WotsSignature, WotsSpongePrivateKeyGeneratorBuilder},
    PrivateKey, PrivateKeyGenerator, PublicKey, RecoverableSignature, Signature,
};
use bee_ternary::{T1B1Buf, TryteBuf};

#[test]
fn invalid_message_length() {
    let message = TryteBuf::try_from_str("CEFLDDLMF9TO9ZNYIDZCTHQDY9ABGGQZHEFTXKWKWZ")
        .unwrap()
        .as_trits()
        .encode::<T1B1Buf>();
    let entropy =
        TryteBuf::try_from_str("CEFLDDLMF9TO9ZLLTYXIPVFIJKAOFRIQLGNYIDZCTDYSWMNXPYNGFAKHQDY9ABGGQZHEFTXKWKWZXEIUD")
            .unwrap()
            .as_trits()
            .encode::<T1B1Buf>();
    let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
        .security_level(WotsSecurityLevel::Medium)
        .build()
        .unwrap();
    let mut private_key = private_key_generator.generate_from_entropy(&entropy).unwrap();

    assert_eq!(
        private_key.sign(&message).err(),
        Some(WotsError::InvalidMessageLength(message.len())),
    );

    let signature = private_key.sign(&entropy).unwrap();

    assert_eq!(
        signature.recover_public_key(&message).err(),
        Some(WotsError::InvalidMessageLength(message.len()))
    );

    let public_key = private_key.generate_public_key().unwrap();

    assert_eq!(
        public_key.verify(&message, &signature).err(),
        Some(WotsError::InvalidMessageLength(message.len()))
    );
}

#[test]
fn invalid_public_key_length() {
    let entropy = TryteBuf::try_from_str("YSWMNXPYNGFAKHQDY9ABGGQZHEFTXKWKWZXEIUD")
        .unwrap()
        .as_trits()
        .encode::<T1B1Buf>();

    assert_eq!(
        WotsPublicKey::<Kerl>::from_trits(entropy.clone()).err(),
        Some(WotsError::InvalidPublicKeyLength(entropy.len()))
    );
}

#[test]
fn invalid_signature_length() {
    let entropy = TryteBuf::try_from_str("YSWMNXPYNGFAKHQDY9ABGGQZHEFTXKWKWZXEIUD")
        .unwrap()
        .as_trits()
        .encode::<T1B1Buf>();

    assert_eq!(
        WotsSignature::<Kerl>::from_trits(entropy.clone()).err(),
        Some(WotsError::InvalidSignatureLength(entropy.len()))
    );
}
