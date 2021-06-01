// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::Packable;

#[derive(Packable)]
#[packable(error = T::Error)]
pub struct Wrap<T: Packable>(T);

fn main() {}
