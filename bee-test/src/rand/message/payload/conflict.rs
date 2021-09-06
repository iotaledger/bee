// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{message::payload::transaction::rand_transaction_id, number::rand_number_range};

use bee_message::payload::fpc::Conflict;

/// Generates a random [`Conflict`].
pub fn rand_conflict() -> Conflict {
    Conflict::new(
        rand_transaction_id(),
        rand_number_range(0..=2),
        rand_number_range(0..=127),
    )
}
