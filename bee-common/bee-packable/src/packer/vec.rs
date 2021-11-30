// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use crate::packer::Packer;

use alloc::vec::Vec;
use core::convert::Infallible;

impl Packer for Vec<u8> {
    type Error = Infallible;

    fn pack_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error> {
        self.extend_from_slice(bytes.as_ref());
        Ok(())
    }
}
