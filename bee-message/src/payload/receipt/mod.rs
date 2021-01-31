// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

pub(crate) const RECEIPT_PAYLOAD_TYPE: u32 = 3;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ReceiptPayload {}

impl ReceiptPayload {
    pub fn new() -> Self {
        Self {}
    }
}

impl Packable for ReceiptPayload {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0
    }

    fn pack<W: Write>(&self, _writer: &mut W) -> Result<(), Self::Error> {
        Ok(())
    }

    fn unpack<R: Read + ?Sized>(_reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}
