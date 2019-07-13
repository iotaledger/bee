//! Core functionality

#![deny(bad_style, missing_docs, unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]

#[macro_use]
pub mod common;

pub mod bee;
pub mod constants;
pub mod errors;
pub mod messaging;

pub use bee::Bee;
