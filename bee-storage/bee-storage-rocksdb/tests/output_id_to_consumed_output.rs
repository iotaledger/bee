// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
mod access;

impl_access_test!(
    output_id_to_consumed_output_access_rocksdb,
    output_id_to_consumed_output_access
);
