// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate std;

use crate::packer::Packer;

use std::io::{self, Write};

impl<W: Write> Packer for W {
    type Error = io::Error;

    fn pack_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error> {
        self.write_all(bytes.as_ref())
    }
}
