// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[allow(deprecated)]
use bee_crypto::ternary::sponge::Kerl;
#[allow(deprecated)]
use bee_signing::ternary::{
    wots::{Error as WotsError, WotsPublicKey, WotsSecurityLevel, WotsSignature, WotsSpongePrivateKeyGeneratorBuilder},
    PrivateKey, PrivateKeyGenerator, PublicKey, RecoverableSignature, Signature,
};
use bee_ternary::{T1B1Buf, TryteBuf};

#[test]
#[allow(deprecated)]
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
    #[allow(deprecated)]
    let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
        .with_security_level(WotsSecurityLevel::Medium)
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
#[allow(deprecated)]
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
#[allow(deprecated)]
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
