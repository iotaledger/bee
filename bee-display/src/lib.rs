//! Display

#![deny(bad_style, missing_docs, unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]

#[macro_use]
extern crate bee_core;

mod constants;

pub mod display;

pub use crate::display::Display;
