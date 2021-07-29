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

const VALID_ADDRESS: [(&str, &str); 8] = [
    (
        "BC1QW508D6QEJXTDG4Y5R3ZARVARY0C5XW7KV8F3T4",
        "0014751e76e8199196d454941c45d1b3a323f1433bd6",
    ),
    (
        "tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7",
        "00201863143c14c5166804bd19203356da136c985678cd4d27a1b8c6329604903262",
    ),
    (
        "bc1pw508d6qejxtdg4y5r3zarvary0c5xw7kw508d6qejxtdg4y5r3zarvary0c5xw7kt5nd6y",
        "5128751e76e8199196d454941c45d1b3a323f1433bd6751e76e8199196d454941c45d1b3a323f1433bd6",
    ),
    ("BC1SW50QGDZ25J", "6002751e"),
    (
        "bc1zw508d6qejxtdg4y5r3zarvaryvaxxpcs",
        "5210751e76e8199196d454941c45d1b3a323",
    ),
    (
        "tb1qqqqqp399et2xygdj5xreqhjjvcmzhxw4aywxecjdzew6hylgvsesrxh6hy",
        "0020000000c4a5cad46221b2a187905e5266362b99d5e91c6ce24d165dab93e86433",
    ),
    (
        "tb1pqqqqp399et2xygdj5xreqhjjvcmzhxw4aywxecjdzew6hylgvsesf3hn0c",
        "5120000000c4a5cad46221b2a187905e5266362b99d5e91c6ce24d165dab93e86433",
    ),
    (
        "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqzk5jj0",
        "512079be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
    ),
];

const INVALID_ADDRESS: [&str; 15] = [
    // Invalid HRP
    "tc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vq5zuyut",
    // Invalid checksum algorithm (bech32 instead of bech32m)
    "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqh2y7hd",
    // Invalid checksum algorithm (bech32 instead of bech32m)
    "tb1z0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqglt7rf",
    // Invalid checksum algorithm (bech32 instead of bech32m)
    "BC1S0XLXVLHEMJA6C4DQV22UAPCTQUPFHLXM9H8Z3K2E72Q4K9HCZ7VQ54WELL",
    // Invalid checksum algorithm (bech32m instead of bech32)
    "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kemeawh",
    // Invalid checksum algorithm (bech32m instead of bech32)
    "tb1q0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vq24jc47",
    // Invalid character in checksum
    "bc1p38j9r5y49hruaue7wxjce0updqjuyyx0kh56v8s25huc6995vvpql3jow4",
    // Invalid witness version
    "BC130XLXVLHEMJA6C4DQV22UAPCTQUPFHLXM9H8Z3K2E72Q4K9HCZ7VQ7ZWS8R",
    // Invalid program length (1 byte)
    "bc1pw5dgrnzv",
    // Invalid program length (41 bytes)
    "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7v8n0nx0muaewav253zgeav",
    // Invalid program length for witness version 0 (per BIP141)
    "BC1QR508D6QEJXTDG4Y5R3ZARVARYV98GJ9P",
    // Mixed case
    "tb1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vq47Zagq",
    // More than 4 padding bits
    "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7v07qwwzcrf",
    // Non-zero padding in 8-to-5 conversion
    "tb1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vpggkg4j",
    // Empty data section
    "bc1gmk9yu",
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
fn check_data() {
    for valid in VALID_ADDRESS {
        println!("{:?}", valid);
        assert_eq!(
            *Bech32Address::new(valid.0).unwrap().data(),
            *hex::decode(valid.1).unwrap()
        );
    }
}
