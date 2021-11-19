// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Cryptographic primitives of the IOTA protocol.

#![deny(clippy::cast_lossless, clippy::checked_conversions)]
#![warn(
    missing_docs,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
#![deprecated(
    note = "`bee-crypto` will not be supported in future versions. You can use functions from `iota-crypto` instead."
)]
#![allow(deprecated)]
pub mod ternary;
