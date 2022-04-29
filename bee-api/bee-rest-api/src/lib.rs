// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Bee REST API

// #![deny(missing_docs, warnings)]

#![cfg_attr(doc_cfg, feature(doc_cfg))]

extern crate bee_message;
extern crate bee_tangle;

#[cfg(feature = "endpoints")]
pub mod endpoints;
pub mod types;
