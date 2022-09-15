// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

use crate::Raw;

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocolParameters {
    pub protocol_version: u8,
    pub params: Raw<bee::protocol::ProtocolParameters>,
}

impl From<proto::RawProtocolParameters> for ProtocolParameters {
    fn from(value: proto::RawProtocolParameters) -> Self {
        Self {
            protocol_version: value.protocol_version as u8,
            params: value.params.into(),
        }
    }
}

impl From<ProtocolParameters> for proto::RawProtocolParameters {
    fn from(value: ProtocolParameters) -> Self {
        Self {
            protocol_version: value.protocol_version as u32,
            params: value.params.data(),
        }
    }
}
