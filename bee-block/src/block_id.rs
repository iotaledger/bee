// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

impl_id!(
    pub BlockId,
    32,
    "A block identifier, the BLAKE2b-256 hash of the block bytes. See <https://www.blake2.net/> for more information."
);

#[cfg(feature = "serde")]
string_serde_impl!(BlockId);
