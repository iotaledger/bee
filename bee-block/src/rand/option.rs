// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::bool::rand_bool;

/// Generates a random generic option.
pub fn rand_option<T>(inner: T) -> Option<T> {
    if rand_bool() { Some(inner) } else { None }
}
