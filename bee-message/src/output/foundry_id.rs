// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

impl_id!(FoundryId, 26, "Defines the unique identifier of a foundry.");

#[cfg(feature = "serde1")]
string_serde_impl!(FoundryId);
