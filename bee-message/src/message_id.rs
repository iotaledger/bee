// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

impl_id!(
    MessageId,
    32,
    "A message identifier, the BLAKE2b-256 hash of the message bytes. See <https://www.blake2.net/> for more information."
);

impl MessageId {
    /// Create a null [`MessageId`].
    pub fn null() -> Self {
        Self([0u8; MessageId::LENGTH])
    }
}
