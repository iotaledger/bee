// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_pow::compute_pow_score;

#[test]
fn score() {
    let message: [u8; 21] = [
        0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21, 0x5e, 0xe6, 0xaa, 0xaa, 0xaa,
        0xaa, 0xaa, 0xaa,
    ];

    assert_eq!(compute_pow_score(&message), 937.2857142857143);

    let message: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

    assert_eq!(compute_pow_score(&message), 3u128.pow(1) as f64 / 8 as f64);

    let message: [u8; 8] = [203, 124, 2, 0, 0, 0, 0, 0];

    assert_eq!(compute_pow_score(&message), 3u128.pow(10) as f64 / 8 as f64);

    let message: [u8; 8] = [65, 235, 119, 85, 85, 85, 85, 85];

    assert_eq!(compute_pow_score(&message), 3u128.pow(14) as f64 / 8 as f64);

    let message: [u8; 10000] = [0; 10000];

    assert_eq!(compute_pow_score(&message), 3u128.pow(0) as f64 / 10000 as f64);
}
