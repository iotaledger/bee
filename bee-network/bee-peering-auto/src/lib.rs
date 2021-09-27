// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Utilities to automatically discover peers.

#![deny(missing_docs)]

mod consts;
mod distance;

#[cfg(feature = "backstage")]
pub mod backstage;
pub mod config;
