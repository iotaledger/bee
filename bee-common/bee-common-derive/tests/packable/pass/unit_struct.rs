// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_common::packable::Packable;

use core::convert::Infallible;

#[derive(Packable)]
#[packable(error = Infallible)]
pub struct Unit;

#[derive(Packable)]
#[packable(error = Infallible)]
pub struct RoundUnit();

fn main() {}
