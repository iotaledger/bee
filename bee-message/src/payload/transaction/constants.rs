// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::Range;

pub const INPUT_OUTPUT_COUNT_MAX: usize = 127;
pub const INPUT_OUTPUT_COUNT_RANGE: Range<usize> = 1..INPUT_OUTPUT_COUNT_MAX + 1;
pub const INPUT_OUTPUT_INDEX_RANGE: Range<u16> = 0..INPUT_OUTPUT_COUNT_MAX as u16;
