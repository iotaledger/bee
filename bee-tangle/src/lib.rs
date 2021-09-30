// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the IOTA tangle.

#![deny(missing_docs)]

mod tangle;
mod walker;

pub use tangle::Tangle;
pub use walker::TangleWalker;
