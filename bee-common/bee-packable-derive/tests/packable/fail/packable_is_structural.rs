// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::Packable;

use core::convert::Infallible;

struct NonPackable;

#[derive(Packable)]
#[packable(error = Infallible)]
pub struct Wrapper(NonPackable);

fn main() {}
