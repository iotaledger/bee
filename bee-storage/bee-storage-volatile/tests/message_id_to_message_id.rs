// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
mod access;

impl_access_test!(
    message_id_to_message_id_access_volatile,
    message_id_to_message_id_access
);
