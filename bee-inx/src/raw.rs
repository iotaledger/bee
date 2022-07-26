// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use bee_block as bee;
use inx::proto;
use packable::{Packable, PackableExt};

use crate::Error;

/// Represents a type as raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raw<T: Packable> {
    data: Vec<u8>,
    phantom: PhantomData<T>,
}

impl<T: Packable> Raw<T> {
    #[must_use]
    pub fn data(self) -> Vec<u8> {
        self.data
    }

    pub fn inner(self, visitor: &mut T::UnpackVisitor) -> Result<T, Error> {
        let unpacked = T::unpack_verified(self.data, visitor)
            .map_err(|e| bee_block::InxError::InvalidRawBytes(format!("{:?}", e)))?;
        Ok(unpacked)
    }
}

impl<T: Packable> From<Vec<u8>> for Raw<T> {
    fn from(value: Vec<u8>) -> Self {
        Self {
            data: value,
            phantom: PhantomData,
        }
    }
}

impl From<proto::RawOutput> for Raw<bee::output::Output> {
    fn from(value: proto::RawOutput) -> Self {
        value.into()
    }
}
