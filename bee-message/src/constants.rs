// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::{Range, RangeInclusive};

/// The total number of IOTA tokens in circulation.
pub const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;

/// The maximum number of outputs for a transaction.
pub const INPUT_OUTPUT_COUNT_MAX: usize = 127;

/// The range of valid numbers of outputs for a transaction [1..127].
pub const INPUT_OUTPUT_COUNT_RANGE: RangeInclusive<usize> = 1..=INPUT_OUTPUT_COUNT_MAX;

/// The range of valid numbers of unlock blocks for a transaction [1..127].
pub const UNLOCK_BLOCK_COUNT_RANGE: RangeInclusive<usize> = INPUT_OUTPUT_COUNT_RANGE;

/// The valid range of indices for inputs and outputs for a transaction [0..126].
pub const INPUT_OUTPUT_INDEX_RANGE: Range<u16> = 0..INPUT_OUTPUT_COUNT_MAX as u16;
