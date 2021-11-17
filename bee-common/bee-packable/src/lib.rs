// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a [`Packable`] trait to serialize and deserialize types.
//!
//! For more information about the design of this crate please read the [`Packable`],
//! [`unpacker`], [`packer`], [`UnpackError`](error::UnpackError) and
//! [`UnpackErrorExt`](error::UnpackErrorExt) documentation.
//!
//! # Motivation
//!
//! This crate was written as a `no_std` replacement for the
//! [`Packable` serialization framework](https://github.com/iotaledger/bee/blob/c08f1bac7170fe5cb650ddc347ac4d483bb9036a/bee-common/bee-common/src/packable.rs)
//! used in the Bee node implementation for Chrysalis Part 2.
//!
//! ## The old `Packable` trait
//!
//! The need for a serialization API existed before Coordicide. Efforts to satisfy this need
//! culminated with the introduction of the `Packable` trait in the `bee-common` crate during the
//! Chrysalis part 2 period. Most of the design decisions behind this crate were done to simplify
//! the serialization of the [IOTA protocol messages](https://github.com/iotaledger/protocol-rfcs/pull/0017).
//! The proposed trait was the following:
//!
//! ```
//! use std::io::{Read, Write};
//!
//! pub trait Packable {
//!     type Error: std::fmt::Debug;
//!
//!     fn packed_len(&self) -> usize;
//!
//!     fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error>;
//!
//!     fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error>
//!     where
//!         Self: Sized;
//! }
//! ```
//! The main issue with this trait is that it cannot be used in a `no_std` environment because it
//! depends explicitly on the [`std::io`] API, whose transition to the [`core`] crate has not been
//! decided yet.  Another issue is that the `Error` type is used to represent three different kinds
//! of errors:
//!
//! - Writing errors: Raised when there are issues while writing bytes.
//! - Reading errors: Raised when there are issues while reading bytes.
//! - Deserialization errors: Raised when the bytes being used to create a value are invalid for
//! the data layout of such value.
//!
//! # Replacing [`std::io`]
//!
//! We introduced the [`Packer`](packer::Packer) and [`Unpacker`](unpacker::Unpacker) taits to
//! abstract away any IO operation without relying on [`std::io`]. This has the additional benefit
//! of allowing us to pack and unpack values from different kinds of buffers.
//!
//! # Types that implement [`Packable`]
//!
//! The [`Packable`] trait is implemented for every integer type by encoding the value as an array
//! of bytes in little-endian order. Booleans are packed following Rust's data layout, meaning that
//! `true` is packed as a `1` byte and `false` as a `0` byte. However, boolean unpacking is less
//! strict and unpacks any non-zero byte as `true`. Additional implementations of [`Packable`] are
//! provided for [`Vec<T>`](std::vec::Vec), `Box<[T]>`, `[T; N]` and [`Option<T>`] if T implements
//! [`Packable`].
//!
//! Check the [`Packable`] `impl` section for further information.

#![no_std]
#![deny(missing_docs)]

#[cfg(doc)]
extern crate std;

mod packable;

pub mod error;
pub mod packer;
pub mod unpacker;

pub use packable::*;
