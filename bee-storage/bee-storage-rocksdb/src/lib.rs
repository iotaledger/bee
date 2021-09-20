// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A bee-storage implementation for the [rocksdb](https://docs.rs/rocksdb/latest/rocksdb/) backend.

#![deny(missing_docs)]

pub mod access;
pub mod column_families;
pub mod compaction;
pub mod compression;
pub mod config;
pub mod error;
pub mod storage;

pub use storage::Storage;
