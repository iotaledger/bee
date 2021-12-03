// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{local::salt::Salt, proto, request::Request};

use base64 as bs64;
use prost::{bytes::BytesMut, DecodeError, EncodeError, Message};

use std::fmt;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Missing salt")]
    MissingSalt,
    #[error("Invalid salt")]
    InvalidSalt,
    #[error("{0}")]
    ProtobufDecode(#[from] DecodeError),
    #[error("{0}")]
    ProtobufEncode(#[from] EncodeError),
}

#[derive(Clone)]
pub(crate) struct PeeringRequest {
    timestamp: u64,
    salt: Salt,
}

impl PeeringRequest {
    pub fn new(salt: Salt) -> Self {
        let timestamp = crate::time::unix_now_secs();

        Self { timestamp, salt }
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn salt(&self) -> &Salt {
        &self.salt
    }

    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, Error> {
        let proto::PeeringRequest { timestamp, salt } = proto::PeeringRequest::decode(bytes)?;
        let proto::Salt { bytes, exp_time } = salt.ok_or(Error::MissingSalt)?;

        Ok(Self {
            timestamp: timestamp as u64,
            salt: Salt {
                bytes: bytes.try_into().map_err(|_| Error::InvalidSalt)?,
                expiration_time: exp_time,
            },
        })
    }

    pub fn to_protobuf(&self) -> Result<BytesMut, Error> {
        let peering_req = proto::PeeringRequest {
            timestamp: self.timestamp as i64,
            salt: Some(proto::Salt {
                bytes: self.salt.bytes().to_vec(),
                exp_time: self.salt.expiration_time(),
            }),
        };

        let mut bytes = BytesMut::with_capacity(peering_req.encoded_len());
        peering_req.encode(&mut bytes)?;

        Ok(bytes)
    }
}

impl fmt::Debug for PeeringRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeeringRequest")
            .field("timestamp", &self.timestamp)
            .field("salt_bytes", &bs64::encode(self.salt().bytes()))
            .field("salt_expiration_time", &self.salt().expiration_time())
            .finish()
    }
}

impl Request for PeeringRequest {}

pub(crate) struct PeeringResponse {
    pub(crate) request_hash: Vec<u8>,
    pub(crate) status: bool,
}

impl PeeringResponse {
    pub fn new(request_hash: Vec<u8>, status: bool) -> Self {
        Self { request_hash, status }
    }

    pub fn request_hash(&self) -> &[u8] {
        &self.request_hash
    }

    pub fn status(&self) -> bool {
        self.status
    }

    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        let proto::PeeringResponse { req_hash, status } = proto::PeeringResponse::decode(bytes)?;

        Ok(Self {
            request_hash: req_hash,
            status,
        })
    }

    pub fn to_protobuf(&self) -> Result<BytesMut, EncodeError> {
        let peering_res = proto::PeeringResponse {
            req_hash: self.request_hash.clone(),
            status: self.status,
        };

        let mut bytes = BytesMut::with_capacity(peering_res.encoded_len());
        peering_res.encode(&mut bytes)?;

        Ok(bytes)
    }
}

impl fmt::Debug for PeeringResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeeringResponse")
            .field("request_hash", &bs58::encode(&self.request_hash).into_string())
            .field("status", &self.status)
            .finish()
    }
}

// NOTE: We don't require a response for `DropRequest`, hence it doesn't need to impl `Request`.
pub(crate) struct DropPeeringRequest {
    pub(crate) timestamp: u64,
}

impl DropPeeringRequest {
    pub fn new() -> Self {
        let timestamp = crate::time::unix_now_secs();

        Self { timestamp }
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        let proto::PeeringDrop { timestamp } = proto::PeeringDrop::decode(bytes)?;

        Ok(Self {
            timestamp: timestamp as u64,
        })
    }

    pub fn to_protobuf(&self) -> Result<BytesMut, EncodeError> {
        let peering_drop = proto::PeeringDrop {
            timestamp: self.timestamp as i64,
        };

        let mut bytes = BytesMut::with_capacity(peering_drop.encoded_len());
        peering_drop.encode(&mut bytes)?;

        Ok(bytes)
    }
}

impl fmt::Debug for DropPeeringRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DropPeeringRequest")
            .field("timestamp", &self.timestamp)
            .finish()
    }
}
