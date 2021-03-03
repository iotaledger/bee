// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::model::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::{milestone::MilestoneIndex, payload::receipt::ReceiptPayload};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    inner: ReceiptPayload,
    included_in: MilestoneIndex,
}

impl Receipt {
    pub fn new(inner: ReceiptPayload, included_in: MilestoneIndex) -> Self {
        Self { inner, included_in }
    }

    pub fn inner(&self) -> &ReceiptPayload {
        &self.inner
    }

    pub fn included_in(&self) -> &MilestoneIndex {
        &self.included_in
    }
}

impl Packable for Receipt {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.inner.packed_len() + self.inner.packed_len() + self.included_in.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.inner.pack(writer)?;
        self.included_in.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(
            ReceiptPayload::unpack(reader)?,
            MilestoneIndex::unpack(reader)?,
        ))
    }
}
