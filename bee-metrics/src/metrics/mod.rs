// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing some predefined useful metrics.

pub mod process;
#[cfg(feature = "sync")]
pub mod sync;

pub use prometheus_client::metrics::{counter, gauge};
