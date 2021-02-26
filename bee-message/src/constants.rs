// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::{Range, RangeInclusive};

pub const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;
pub const INPUT_OUTPUT_COUNT_MAX: usize = 127;
pub const INPUT_OUTPUT_COUNT_RANGE: RangeInclusive<usize> = 1..=INPUT_OUTPUT_COUNT_MAX;
pub const INPUT_OUTPUT_INDEX_RANGE: Range<u16> = 0..INPUT_OUTPUT_COUNT_MAX as u16;
