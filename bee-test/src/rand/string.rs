// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rand::{distributions::Alphanumeric, thread_rng, Rng};

pub fn rand_string(len: usize) -> String {
    String::from_utf8(thread_rng().sample_iter(&Alphanumeric).take(len).collect()).unwrap()
}
