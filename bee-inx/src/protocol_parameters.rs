// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::protocol::ProtocolParameters;
use inx::proto;
use packable::PackableExt;

use crate::Error;

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawProtocolParameters {
    pub protocol_version: u8,
    pub params: Vec<u8>,
}

impl RawProtocolParameters {
    pub fn inner(self) -> Result<ProtocolParameters, Error> {
        let unpacked = ProtocolParameters::unpack_verified(self.params)
            .map_err(|e| bee_block::InxError::InvalidRawBytes(format!("{:?}", e)))?;
        Ok(unpacked)
    }
}

impl From<proto::RawProtocolParameters> for RawProtocolParameters {
    fn from(value: proto::RawProtocolParameters) -> Self {
        Self {
            protocol_version: value.protocol_version as u8,
            params: value.params,
        }
    }
}
