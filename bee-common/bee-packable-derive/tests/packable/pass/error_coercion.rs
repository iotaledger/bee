// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::{
    error::{PackError, UnknownTagError, UnpackError},
    packer::Packer,
    unpacker::Unpacker,
    coerce::CoerceInfallible,
    Packable,
};

use core::convert::Infallible;

pub struct Picky(u8);

pub struct PickyError(u8);

impl Packable for Picky {
    type PackError = Infallible;
    type UnpackError = PickyError;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.0.pack(packer)
    }

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let value = u8::unpack(unpacker).infallible()?;

        if value == 42 {
            Ok(Self(value))
        } else {
            Err(UnpackError::Packable(PickyError(value)))
        }
    }
}

pub enum PickyOrByteError {
    Picky(PickyError),
    UnknownTag(u8),
}

impl From<Infallible> for PickyOrByteError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl From<PickyError> for PickyOrByteError {
    fn from(err: PickyError) -> Self {
        Self::Picky(err)
    }
}

impl From<UnknownTagError<u8>> for PickyOrByteError {
    fn from(err: UnknownTagError<u8>) -> Self {
        Self::UnknownTag(err.0)
    }
}

#[derive(Packable)]
#[packable(tag_type = u8)]
#[packable(pack_error = Infallible)]
#[packable(unpack_error = PickyOrByteError)]
pub enum PickyOrByte {
    #[packable(tag = 0)]
    Picky(Picky),
    #[packable(tag = 1)]
    Byte(u8),
}

pub struct PickyAndByteError(PickyError);

impl From<Infallible> for PickyAndByteError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl From<PickyError> for PickyAndByteError {
    fn from(err: PickyError) -> Self {
        Self(err)
    }
}

#[derive(Packable)]
#[packable(pack_error = Infallible)]
#[packable(unpack_error = PickyAndByteError)]
pub struct PickyAndByte {
    picky: Picky,
    byte: u8,
}

fn main() {}
