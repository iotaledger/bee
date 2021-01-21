// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Divisor used to compute the allowed dust outputs on an address.
/// The amount of dust outputs on an address is calculated by:
/// sum(dust_allowance_output_deposit) / DUST_ALLOWANCE_DIVISOR.
/// Example: 1_000_000 / 10_000 = 100 dust outputs.
pub const DUST_ALLOWANCE_DIVISOR: u64 = 10_000;

/// Minimum deposit amount.
pub const DUST_ALLOWANCE_MINIMUM: u64 = 1_000_000;
