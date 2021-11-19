// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Signing scheme primitives.

#![allow(deprecated)] // This is ok, because we are going to deprecate everything here anyways.
#![deny(clippy::cast_lossless, clippy::checked_conversions)]
#![warn(
    missing_docs,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

#[deprecated(note = "`bee-signing` will no longer be supported.")]
pub mod ternary;
