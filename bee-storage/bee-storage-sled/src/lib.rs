// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A bee-storage implementation for the [sled](https://docs.rs/sled/latest/sled/) backend.

#![deny(missing_docs, warnings)]

pub mod access;
pub mod config;
pub mod error;
pub mod storage;
pub mod trees;

pub use storage::Storage;
