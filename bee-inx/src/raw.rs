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
    _phantom: PhantomData<T>,
}

impl<T: Packable> Raw<T> {
    #[must_use]
    pub fn data(self) -> Vec<u8> {
        self.data
    }

    pub fn inner(self, visitor: &T::UnpackVisitor) -> Result<T, Error> {
        let unpacked = T::unpack_verified(self.data, visitor)
            .map_err(|e| bee_block::InxError::InvalidRawBytes(format!("{:?}", e)))?;
        Ok(unpacked)
    }
}

impl<T: Packable> From<Vec<u8>> for Raw<T> {
    fn from(value: Vec<u8>) -> Self {
        Self {
            data: value,
            _phantom: PhantomData,
        }
    }
}

impl From<proto::RawOutput> for Raw<bee::output::Output> {
    fn from(value: proto::RawOutput) -> Self {
        value.data.into()
    }
}

impl From<Raw<bee::output::Output>> for proto::RawOutput {
    fn from(value: Raw<bee::output::Output>) -> Self {
        Self { data: value.data }
    }
}

impl From<proto::RawBlock> for Raw<bee::Block> {
    fn from(value: proto::RawBlock) -> Self {
        value.data.into()
    }
}

impl From<Raw<bee::Block>> for proto::RawBlock {
    fn from(value: Raw<bee::Block>) -> Self {
        Self { data: value.data }
    }
}

impl From<proto::RawMilestone> for Raw<bee::payload::milestone::MilestonePayload> {
    fn from(value: proto::RawMilestone) -> Self {
        value.data.into()
    }
}

impl From<Raw<bee::payload::milestone::MilestonePayload>> for proto::RawMilestone {
    fn from(value: Raw<bee::payload::milestone::MilestonePayload>) -> Self {
        Self { data: value.data }
    }
}

#[cfg(test)]
mod test {
    use bee::rand::output::rand_output;

    use super::*;
    use crate::ProtocolParameters;

    #[test]
    fn raw_output() {
        let protocol_parameters = bee::protocol::ProtocolParameters::default();

        let output = rand_output(&protocol_parameters);

        let proto = proto::RawOutput {
            data: output.pack_to_vec(),
        };
        let raw: Raw<bee::output::Output> = proto.into();
        assert_eq!(output, raw.inner(&protocol_parameters).unwrap());
    }

    #[test]
    fn raw_protocol_parameters() {
        let protocol_parameters = bee::protocol::protocol_parameters();
        let proto = proto::RawProtocolParameters::from(protocol_parameters.clone());

        let pp: ProtocolParameters = proto.into();
        assert_eq!(protocol_parameters, pp.params.inner(&()).unwrap());
    }
}
