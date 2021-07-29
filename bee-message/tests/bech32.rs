// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{address::Bech32Address, error::ValidationError};

use core::convert::TryFrom;

const VALID_BECH32: [(&str, &str); 6] = [
    // ("A12UEL5L", "A"),
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
