// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
mod access;

impl_access_test!(
    milestone_index_to_milestone_access_rocksdb,
    milestone_index_to_milestone_access
);
