// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A general purpose storage backend crate with key value abstraction API.
//!
//! # Features
//!
//! ## Access implementation:
//! - Traits contracts which define the general purpose database operations such as (insert, fetch, ...);
//! ## Backend implementation:
//! - Trait contract to start and shutdown backends;
//! - Configuration and associated builder to configure different backends;
//!
//! This crate tries to simplify the implementation of various storage backends and provides unified access API for the
//! application/user space.

#![deny(missing_docs)]
#![deny(warnings)]

pub mod access;
pub mod backend;
pub mod system;
