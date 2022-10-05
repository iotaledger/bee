// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![no_main]

use bee_block::signature::Signature;

use libfuzzer_sys::fuzz_target;
use packable::PackableExt;

fuzz_target!(|data: &[u8]| {
    let _ = Signature::unpack_verified(data.to_vec().as_slice());
});
