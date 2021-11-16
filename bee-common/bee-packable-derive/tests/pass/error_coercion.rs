// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::{
    error::UnpackErrorExt,
    error::{UnknownTagError, UnpackError},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use core::convert::Infallible;

pub struct Picky(u8);

#[derive(Debug)]
pub struct PickyError(u8);

impl Packable for Picky {
    type UnpackError = PickyError;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.0.pack(packer)
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let value = u8::unpack::<_, VERIFY>(unpacker).infallible()?;

        if value == 42 {
            Ok(Self(value))
        } else {
            Err(UnpackError::Packable(PickyError(value)))
        }
    }
}

#[derive(Debug)]
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
#[packable(unpack_error = PickyOrByteError)]
pub enum PickyOrByte {
    #[packable(tag = 0)]
    Picky(Picky),
    #[packable(tag = 1)]
    Byte(u8),
}

#[derive(Debug)]
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
#[packable(unpack_error = PickyAndByteError)]
pub struct PickyAndByte {
    picky: Picky,
    byte: u8,
}

fn main() {}
