// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that provides types and workers enabling the IOTA protocol.

// TODO
// #![deny(missing_docs, warnings)]

pub mod types;
#[cfg(feature = "workers")]
pub mod workers;
