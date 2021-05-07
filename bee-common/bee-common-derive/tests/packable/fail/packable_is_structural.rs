// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_common::packable::Packable;
use bee_common_derive::Packable;

struct NonPackable;

#[derive(Packable)]
pub struct Wrapper(NonPackable);

fn main() {}
