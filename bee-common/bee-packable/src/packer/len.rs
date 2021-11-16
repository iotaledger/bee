// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::packer::Packer;

#[repr(transparent)]
pub(crate) struct LenPacker(pub(crate) usize);

impl Packer for LenPacker {
    type Error = core::convert::Infallible;

    #[inline(always)]
    fn pack_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error> {
        self.0 += bytes.as_ref().len();

        Ok(())
    }
}
