// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_pow::score::compute_pow_score;

// Tests are from:
// https://github.com/Wollac/protocol-rfcs/blob/message-pow/text/0024-message-pow/0024-message-pow.md#example
// https://github.com/Wollac/iota-crypto-demo/blob/master/pkg/pow/pow_test.go#L26

#[test]
fn score() {
    let message: [u8; 21] = [
        0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21, 0x5e, 0xe6, 0xaa, 0xaa, 0xaa,
        0xaa, 0xaa, 0xaa,
    ];

    assert!((compute_pow_score(&message) - 937.2857142857143).abs() < f64::EPSILON);

    let message: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

    assert!((compute_pow_score(&message) - 3u128.pow(1) as f64 / 8_f64).abs() < f64::EPSILON);

    let message: [u8; 8] = [203, 124, 2, 0, 0, 0, 0, 0];

    assert!((compute_pow_score(&message) - 3u128.pow(10) as f64 / 8_f64).abs() < f64::EPSILON);

    let message: [u8; 8] = [65, 235, 119, 85, 85, 85, 85, 85];

    assert!((compute_pow_score(&message) - 3u128.pow(14) as f64 / 8_f64).abs() < f64::EPSILON);

    let message: [u8; 10000] = [0; 10000];

    assert!((compute_pow_score(&message) - 3u128.pow(0) as f64 / 10000_f64).abs() < f64::EPSILON);
}
