// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::Packable;

#[derive(Packable)]
#[packable(pack_error = T::PackError)]
#[packable(unpack_error = T::UnpackError)]
pub struct Wrap<T: Packable>(T);

fn main() {}
