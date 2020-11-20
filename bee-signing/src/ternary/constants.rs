// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_crypto::ternary::HASH_LENGTH;

/// Length of a message fragment.
pub const MESSAGE_FRAGMENT_LENGTH: usize = 27;

/// Length of a signature fragment.
pub const SIGNATURE_FRAGMENT_LENGTH: usize = MESSAGE_FRAGMENT_LENGTH * HASH_LENGTH;
