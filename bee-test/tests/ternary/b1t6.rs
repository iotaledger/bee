// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ternary::{b1t6, T1B1Buf};

// TODO factorize tests

#[test]
fn encode() {
    let bytes = vec![1u8];
    let str = b1t6::encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(char::from)
        .collect::<String>();

    assert_eq!(str, "A9");

    let bytes = vec![127u8];
    let str = b1t6::encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(char::from)
        .collect::<String>();

    assert_eq!(str, "SE");

    let bytes = vec![128u8];
    let str = b1t6::encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(char::from)
        .collect::<String>();

    assert_eq!(str, "GV");

    let bytes = vec![255u8];
    let str = b1t6::encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(char::from)
        .collect::<String>();

    assert_eq!(str, "Z9");

    let bytes = vec![0u8, 1u8];
    let str = b1t6::encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(char::from)
        .collect::<String>();

    assert_eq!(str, "99A9");

    let bytes = vec![
        0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8,
        0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8,
        0u8, 1u8, 0u8, 1u8, 0u8, 1u8,
    ];
    let str = b1t6::encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(char::from)
        .collect::<String>();

    assert_eq!(
        str,
        "99A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A9"
    );

    let bytes = hex::decode("0001027e7f8081fdfeff").unwrap();
    let str = b1t6::encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(char::from)
        .collect::<String>();

    assert_eq!(str, "99A9B9RESEGVHVX9Y9Z9");

    let bytes = hex::decode("9ba06c78552776a596dfe360cc2b5bf644c0f9d343a10e2e71debecd30730d03").unwrap();
    let str = b1t6::encode::<T1B1Buf>(&bytes)
        .iter_trytes()
        .map(char::from)
        .collect::<String>();

    assert_eq!(str, "GWLW9DLDDCLAJDQXBWUZYZODBYPBJCQ9NCQYT9IYMBMWNASBEDTZOYCYUBGDM9C9");
}

#[test]
fn encode_decode() {
    let bytes = [
        111, 158, 133, 16, 184, 139, 14, 164, 251, 198, 132, 223, 144, 186, 49, 5, 64, 55, 10, 4, 3, 6, 123, 34, 206,
        244, 151, 31, 236, 62, 139, 184, 187, 7, 40, 131,
    ];
    let encoded = b1t6::encode::<T1B1Buf>(&bytes);
    let decoded = b1t6::decode(&encoded).unwrap();
    assert_eq!(bytes, decoded[..]);
}
