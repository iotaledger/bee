// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A general-purpose Storage Backend crate, with key value abstraction API.
//!
//! # Features
//!
//! ## Backend implementation:
//! - Trait contract to start and shutdown backends
//! - Config and associated builder to configure different backends
//! ## Access implementation:
//! - Traits contracts which define the general-purpose db operations such as (insert, fetch, etc)
//!
//! This crate tries to simplify the implementation of various backends
//! and provides unified Access API for the application/user space.

#![deny(missing_docs)]
/// Access module which form the access layer of the backend
/// which holds the contract of unified database access operations across all the backends and bee types
pub mod access;
/// Storage module which form the backend layer of the backend
/// which holds the contract of starting and shutting down the backend
pub mod storage;
