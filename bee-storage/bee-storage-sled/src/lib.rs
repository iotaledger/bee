// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Bee storage backend using [sled](https://sled.rs).

#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![deny(missing_docs)]
#![deny(warnings)]

pub mod access;
pub mod config;
pub mod storage;
pub mod trees;
