// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that contains the pruning logic.

mod batch;
mod error;
mod metrics;

pub(crate) mod condition;
pub(crate) mod prune;

pub mod config;
