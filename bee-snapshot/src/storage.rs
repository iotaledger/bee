// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_storage::storage;

pub trait Backend: storage::Backend {}

impl<T> Backend for T where T: storage::Backend {}
