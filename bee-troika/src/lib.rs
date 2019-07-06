//! Troika implementations for Bee.
//!
//! This crate currently contains two Troika implementations:
//! * Troika (standard)
//! * F-Troika

#![deny(bad_style, missing_docs, unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]

mod constants;
mod types;

mod ftroika;
#[cfg(feature = "troika")]
mod troika;

pub use ftroika::ftroika::Ftroika;
#[cfg(feature = "troika")]
pub use troika::troika::Troika;
