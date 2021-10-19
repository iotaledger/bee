// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that provides common functionalities shared across multiple crates within the Bee framework, and for
//! applications built on-top.

#![warn(missing_docs)]

#[cfg(feature = "auth")]
pub mod auth;
pub mod logger;
pub mod ord;
pub mod packable;
pub mod time;
