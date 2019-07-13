//! This crate contains types as specified by the IOTA protocol.

#![deny(
    bad_style,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(not(debug_assertions), deny(warnings))]

pub mod time;

#[cfg(not(feature = "constants"))]
mod constants;
#[cfg(any(feature = "constants", feature = "all"))]
pub mod constants;

#[cfg(not(feature = "types"))]
mod transaction;
#[cfg(any(feature = "types", feature = "all"))]
pub mod transaction;

#[cfg(any(feature = "types", feature = "all"))]
pub use crate::transaction::Transaction;
pub use crate::transaction::TransactionBuilder;
