// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::proto;

use base64 as bs64;
use prost::{bytes::BytesMut, DecodeError, EncodeError, Message};

use std::fmt;

pub(crate) struct PeeringRequest(proto::PeeringRequest);

impl PeeringRequest {
    pub fn new(salt_bytes: Vec<u8>, salt_expiration_time: u64) -> Self {
        let timestamp = crate::timestamp::timestamp();

        Self(proto::PeeringRequest {
            timestamp,
            salt: Some(proto::Salt {
                bytes: salt_bytes,
                exp_time: salt_expiration_time,
            }),
        })
    }

    pub fn timestamp(&self) -> i64 {
        self.0.timestamp
    }

    pub fn salt_bytes(&self) -> &Vec<u8> {
        &self.0.salt.as_ref().expect("missing salt").bytes
    }

    pub fn salt_expiration_time(&self) -> u64 {
        self.0.salt.as_ref().expect("missing salt").exp_time
    }

    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(proto::PeeringRequest::decode(bytes)?))
    }

    pub fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        let mut buf = BytesMut::with_capacity(self.0.encoded_len());
        self.0.encode(&mut buf)?;

        Ok(buf)
    }
}

impl fmt::Debug for PeeringRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeeringRequest")
            .field("timestamp", &self.0.timestamp)
            .field(
                "salt_bytes",
                &bs64::encode(&self.0.salt.as_ref().expect("missing salt").bytes),
            )
            .field(
                "salt_expiration_time",
                &self.0.salt.as_ref().expect("missing salt").exp_time,
            )
            .finish()
    }
}

pub(crate) struct PeeringResponse(proto::PeeringResponse);

impl PeeringResponse {
    pub fn new(req_data: &[u8], status: bool) -> Self {
        let res = proto::PeeringResponse {
            req_hash: crate::packets::packet_hash(req_data),
            status,
        };

        Self(res)
    }

    pub fn req_hash(&self) -> &Vec<u8> {
        &self.0.req_hash
    }

    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        let res = proto::PeeringResponse::decode(bytes)?;

        Ok(Self(res))
    }

    pub fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        let mut bytes = BytesMut::with_capacity(self.0.encoded_len());
        self.0.encode(&mut bytes)?;

        Ok(bytes)
    }
}

impl fmt::Debug for PeeringResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeeringResponse")
            .field("req_hash", &bs58::encode(&self.0.req_hash).into_string())
            .field("status", &self.0.status)
            .finish()
    }
}

pub(crate) struct PeeringDrop(proto::PeeringDrop);

impl PeeringDrop {
    pub fn new() -> Self {
        let timestamp = crate::timestamp::timestamp();

        Self(proto::PeeringDrop { timestamp })
    }

    pub fn timestamp(&self) -> i64 {
        self.0.timestamp
    }

    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(proto::PeeringDrop::decode(bytes)?))
    }

    pub fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        let mut buf = BytesMut::with_capacity(self.0.encoded_len());
        self.0.encode(&mut buf)?;

        Ok(buf)
    }
}

impl fmt::Debug for PeeringDrop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeeringDrop")
            .field("timestamp", &self.0.timestamp)
            .finish()
    }
}
