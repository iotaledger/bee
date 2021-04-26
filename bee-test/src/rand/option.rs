// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rand::Rng;

pub fn rand_option<T>(inner: T) -> Option<T> {
    if rand::thread_rng().gen::<f64>() < 0.5 {
        None
    } else {
        Some(inner)
    }
}
