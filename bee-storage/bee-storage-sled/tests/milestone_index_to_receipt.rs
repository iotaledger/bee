// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
mod access;

impl_access_test!(
    milestone_index_to_receipt_access_sled,
    milestone_index_to_receipt_access
);
