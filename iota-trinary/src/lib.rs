#![no_std]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate failure;

pub mod trytes;

pub use self::trytes::*;

type Result<T> = ::core::result::Result<T, failure::Error>;
