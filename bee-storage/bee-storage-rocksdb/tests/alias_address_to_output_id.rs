// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
mod access;

impl_access_test!(
    alias_address_to_output_id_access_rocksdb,
    alias_address_to_output_id_access
);
