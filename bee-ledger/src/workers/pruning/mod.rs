// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that contains the pruning logic.

mod batch;
mod metrics;

pub(crate) mod condition;
pub(crate) mod error;
pub(crate) mod prune;

pub mod config;
