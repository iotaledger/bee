// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that contains all types and features required to compute and maintain the ledger state.
//!
//! ## Feature Flags
//! - `cpt2`: Enable support for backwards compatible output and transaction payload types.

#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![deny(missing_docs)]

pub mod types;
#[cfg(feature = "workers")]
pub mod workers;
