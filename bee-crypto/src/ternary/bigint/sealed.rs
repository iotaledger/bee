// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Sealed trait to protect against downstream implementations.
//! https://rust-lang.github.io/api-guidelines/future-proofing.html.

pub trait Sealed {}
