//! This library provides certain functions to convert between trits, trytes, bytes, tryte
//! strings, ASCII text and signed 64 bit numbers.
//!
//! # Features
//! * `no-std` (with `liballoc` dependency)
//! * `no-checks` compile feature if consumer of this library already ensures valid
//!   inputs.
//! * 9 Trits per 2 Bytes (9/2) byte encoding
//! * no unsafe code

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    //unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(test)]
extern crate test;

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[macro_use]
extern crate lazy_static;

#[cfg(not(feature = "std"))]
pub(crate) mod std {
    pub use core::*;
}

mod constants;
mod luts;

pub mod ascii_strings;
pub mod bytes;
pub mod numbers;
pub mod trits;
pub mod tryte_strings;
pub mod trytes;
pub mod types;
pub mod util;
