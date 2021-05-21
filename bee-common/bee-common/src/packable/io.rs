// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate std;

use super::{Packer, Unpacker};

use std::io::{self, Read, Write};

impl<W: Write> Packer for W {
    type Error = io::Error;

    fn pack_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error> {
        self.write_all(bytes.as_ref())
    }
}

impl<R: Read> Unpacker for R {
    type Error = io::Error;

    fn unpack_bytes<B: AsMut<[u8]>>(&mut self, mut bytes: B) -> Result<(), Self::Error> {
        self.read_exact(bytes.as_mut())
    }
}
