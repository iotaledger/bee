// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// #![deny(missing_docs, warnings)]

//! Bee REST API
//!
//! ## Feature Flags
//! - `cpt2`: Enable support for backwards compatible output and transaction payload types.

#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[cfg(feature = "endpoints")]
pub mod endpoints;
pub mod types;
