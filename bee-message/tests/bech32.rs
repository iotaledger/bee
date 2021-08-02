// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{address::Bech32Address, error::ValidationError};

use core::convert::TryFrom;

const VALID_BECH32: [(&str, &str); 7] = [
    ("A12UEL5L", "a"),
    ("a12uel5l", "a"),
    (
        "an83characterlonghumanreadablepartthatcontainsthenumber1andtheexcludedcharactersbio1tt5tgs",
        "an83characterlonghumanreadablepartthatcontainsthenumber1andtheexcludedcharactersbio",
    ),
    ("abcdef1qpzry9x8gf2tvdw0s3jn54khce6mua7lmqqqxw", "abcdef"),
    (
        "11qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqc8247j",
        "1",
    ),
    ("split1checkupstagehandshakeupstreamerranterredcaperred2y9e3w", "split"),
    ("?1ezyfcl", "?"),
];

const INVALID_BECH32: [&str; 9] = [
    // HRP character out of range
    " 1nwldj5",
    // Overall max length exceeded
    "an84characterslonghumanreadablepartthatcontainsthenumber1andtheexcludedcharactersbio1569pvx",
    // No separator character
    "pzry9x0s0muk",
    // Empty HRP
    "1pzry9x0s0muk",
    // Invalid data character
    "x1b4n0q5v",
    // Too short checksum
    "li1dgmt3",
    // Checksum calculated with uppercase form of HRP
    "A1G7SGD8",
    // Empty HRP
    "10a06t8",
    // Empty HRP
    "1qzzfhee",
];

const VALID_ADDRESSES: [(&str, &str, &str); 2] = [
    (
        "iota1qrhacyfwlcnzkvzteumekfkrrwks98mpdm37cj4xx3drvmjvnep6xqgyzyx",
        "iota",
        "efdc112efe262b304bcf379b26c31bad029f616ee3ec4aa6345a366e4c9e43a3",
    ),
    (
        "atoi1qrhacyfwlcnzkvzteumekfkrrwks98mpdm37cj4xx3drvmjvnep6x8x4r7t",
        "atoi",
        "efdc112efe262b304bcf379b26c31bad029f616ee3ec4aa6345a366e4c9e43a3",
    ),
];

#[test]
fn valid_bech32() {
    for valid in VALID_BECH32 {
        assert!(Bech32Address::new(valid.0).is_ok());
        assert!(Bech32Address::try_from(valid.0).is_ok());
        assert!(Bech32Address::try_from(valid.0.to_string()).is_ok());
    }
}

#[test]
fn invalid_bech32() {
    for invalid in INVALID_BECH32 {
        assert!(matches!(
            Bech32Address::new(invalid),
            Err(ValidationError::InvalidAddress)
        ));
        assert!(matches!(
            Bech32Address::try_from(invalid),
            Err(ValidationError::InvalidAddress)
        ));
        assert!(matches!(
            Bech32Address::try_from(invalid.to_string()),
            Err(ValidationError::InvalidAddress)
        ));
    }
}

#[test]
fn check_hrp() {
    for valid in VALID_BECH32 {
        assert_eq!(Bech32Address::new(valid.0).unwrap().hrp(), valid.1);
    }
}

#[test]
fn valid_addresses() {
    for valid in VALID_ADDRESSES {
        let bech32 = Bech32Address::new(valid.0).unwrap();
        assert_eq!(bech32.hrp(), valid.1);
        assert_eq!(bech32.data()[1..], hex::decode(valid.2).unwrap());
    }
}

// #[test]
// fn generate_bech32_string() {
//     let address = Address::from(Ed25519Address::new(hex_decode(ED25519_ADDRESS).unwrap()));
//     let bech32 = Bech32Address::from_address("iota", &address);
//
//     assert_eq!(
//         bech32.to_string(),
//         "iota1qpf0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryj430ldu"
//     );
// }
//
// #[test]
// fn generate_bech32_testnet_string() {
//     let address = Address::from(Ed25519Address::new(hex_decode(ED25519_ADDRESS).unwrap()));
//     let bech32 = Bech32Address::from_address("atoi", &address);
//
//     assert_eq!(
//         bech32.to_string(),
//         "atoi1qpf0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryjjl77h3"
//     );
// }
//
// #[test]
// fn bech32_to_address() {
//     let address = Address::from(Ed25519Address::new(hex_decode(ED25519_ADDRESS).unwrap()));
//
//     let address: Address = Bech32Address::from_address("iota", &address).try_into().unwrap();
//     assert_eq!(address.to_string(), ED25519_ADDRESS);
// }
