// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate std;

use super::{Packer, Unpacker};

use std::io::{self, Read, Write};

impl<W: Write> Packer for W {
    type Error = io::Error;

    fn pack_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write_all(bytes)
    }
}

impl<R: Read> Unpacker for R {
    type Error = io::Error;

    fn unpack_bytes(&mut self, slice: &mut [u8]) -> Result<(), Self::Error> {
        self.read_exact(slice)
    }
}
