// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Bee REST API

// #![deny(missing_docs, warnings)]

#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[cfg(feature = "endpoints")]
pub mod endpoints;
pub mod types;
