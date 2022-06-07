// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![no_main]

use bee_block::output::Feature;

use libfuzzer_sys::fuzz_target;
use packable::PackableExt;

fuzz_target!(|data: &[u8]| {
    let _ = Feature::unpack_verified(&mut data.to_vec().as_slice());
});
