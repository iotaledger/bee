// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rand::Rng;

pub fn rand_bool() -> bool {
    if rand::thread_rng().gen::<f64>() < 0.5 {
        false
    } else {
        true
    }
}
