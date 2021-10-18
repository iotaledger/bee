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

// TODO check if still needed.

/// Amount of tokens below which an output is considered a dust output.
pub const DUST_THRESHOLD: u64 = 1_000_000;
/// Divisor used to compute the allowed dust outputs on an address.
pub const DUST_ALLOWANCE_DIVISOR: u64 = 100_000;
/// Maximum number of dust outputs for an address.
pub const DUST_OUTPUTS_MAX: u64 = 100;

/// The maximum number of allowed dust outputs on an address is `dust_allowance_sum` divided by `DUST_ALLOWANCE_DIVISOR`
/// and rounded down, i.e. 10 outputs for each 1 Mi deposited. `dust_allowance_sum` is the sum of all the amounts of all
/// unspent `SigLockedDustAllowanceOutputs` on this address. Regardless of `dust_allowance_sum`, the number of dust
/// outputs must never exceed `DUST_OUTPUTS_MAX` per address.
pub fn dust_outputs_max(dust_allowance_sum: u64) -> u64 {
    DUST_OUTPUTS_MAX.min(dust_allowance_sum / DUST_ALLOWANCE_DIVISOR)
}
