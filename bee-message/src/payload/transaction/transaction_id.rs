// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

impl_id!(
    TransactionId,
    32,
    "A transaction identifier, the BLAKE2b-256 hash of the transaction bytes. See <https://www.blake2.net/> for more information."
);

#[cfg(feature = "serde1")]
string_serde_impl!(TransactionId);
