// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::UnpackError, packer::Packer, unpacker::Unpacker, Packable};

use primitive_types::U256;

use core::convert::Infallible;

impl Packable for U256 {
    type UnpackError = Infallible;

    #[inline(always)]
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.0.pack(packer)
    }

    #[inline(always)]
    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        <[u64; 4]>::unpack::<_, VERIFY>(unpacker).map(Self)
    }
}
