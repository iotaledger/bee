// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ternary::{b1t6::encode, T1B1Buf};

// TODO factorize tests

#[test]
fn decode() {
    let bytes = vec![1u8];
    let str = encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "A9");

    let bytes = vec![127u8];
    let str = encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "SE");

    let bytes = vec![128u8];
    let str = encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "GV");

    let bytes = vec![255u8];
    let str = encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "Z9");

    let bytes = vec![0u8, 1u8];
    let str = encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "99A9");

    let bytes = vec![
        0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8,
        0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8,
        0u8, 1u8, 0u8, 1u8, 0u8, 1u8,
    ];
    let str = encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(
        str,
        "99A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A9"
    );

    let bytes = hex::decode("0001027e7f8081fdfeff").unwrap();
    let str = encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "99A9B9RESEGVHVX9Y9Z9");

    let bytes = hex::decode("9ba06c78552776a596dfe360cc2b5bf644c0f9d343a10e2e71debecd30730d03").unwrap();
    let str = encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "GWLW9DLDDCLAJDQXBWUZYZODBYPBJCQ9NCQYT9IYMBMWNASBEDTZOYCYUBGDM9C9");
}
