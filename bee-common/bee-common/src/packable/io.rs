// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate std;

use super::{Packer, UnpackError, Unpacker};

use std::{
    io::{self, Read, Write},
    string::{String, ToString},
};

#[derive(Debug)]
pub enum IoError {
    Io(io::Error),
    Message(String),
}

impl UnpackError for IoError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

impl From<io::Error> for IoError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl<W: Write> Packer for W {
    type Error = IoError;

    fn pack_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        Ok(self.write_all(bytes)?)
    }
}

impl<R: Read> Unpacker for R {
    type Error = IoError;

    fn unpack_bytes(&mut self, slice: &mut [u8]) -> Result<(), Self::Error> {
        Ok(self.read_exact(slice)?)
    }
}
