//! This library allows fast trimming of IOTA transactions to spare bandwidth and storage.

#![deny(
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

#[cfg(feature = "trim_data")]
pub mod trim_data;

#[cfg(feature = "trim_full")]
pub mod trim_full;
