// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{local::salt::Salt, proto, request::Request};

use base64 as bs64;
use crypto::hashes::sha::SHA256_LEN;
use prost::{bytes::BytesMut, DecodeError, EncodeError, Message};

use std::fmt;

#[derive(Clone)]
pub(crate) struct PeeringRequest {
    timestamp: u64,
    salt: Salt,
}

impl PeeringRequest {
    pub(crate) fn new(salt: Salt) -> Self {
        let timestamp = crate::time::unix_now_secs();

        Self { timestamp, salt }
    }

    pub(crate) fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub(crate) fn salt(&self) -> &Salt {
        &self.salt
    }

    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, Error> {
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

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_protobuf(&self) -> BytesMut {
        let peering_req = proto::PeeringRequest {
            timestamp: self.timestamp as i64,
            salt: Some(proto::Salt {
                bytes: self.salt.bytes().to_vec(),
                exp_time: self.salt.expiration_time(),
            }),
        };

        let mut bytes = BytesMut::with_capacity(peering_req.encoded_len());

        // Panic: we have allocated a properly sized buffer.
        peering_req.encode(&mut bytes).expect("encoding peering request failed");

        bytes
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
    request_hash: [u8; SHA256_LEN],
    status: bool,
}

impl PeeringResponse {
    pub(crate) fn new(request_hash: [u8; SHA256_LEN], status: bool) -> Self {
        Self { request_hash, status }
    }

    pub(crate) fn request_hash(&self) -> &[u8] {
        &self.request_hash
    }

    pub(crate) fn status(&self) -> bool {
        self.status
    }

    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, Error> {
        let proto::PeeringResponse { req_hash, status } = proto::PeeringResponse::decode(bytes)?;

        Ok(Self {
            request_hash: req_hash.try_into().map_err(|_| Error::RestoreRequestHash)?,
            status,
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_protobuf(&self) -> BytesMut {
        let peering_res = proto::PeeringResponse {
            req_hash: self.request_hash.to_vec(),
            status: self.status,
        };

        let mut bytes = BytesMut::with_capacity(peering_res.encoded_len());

        // Panic: we have allocated a properly sized buffer.
        peering_res
            .encode(&mut bytes)
            .expect("encoding peering response failed");

        bytes
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
    pub(crate) fn new() -> Self {
        let timestamp = crate::time::unix_now_secs();

        Self { timestamp }
    }

    pub(crate) fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        let proto::PeeringDrop { timestamp } = proto::PeeringDrop::decode(bytes)?;

        Ok(Self {
            timestamp: timestamp as u64,
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_protobuf(&self) -> BytesMut {
        let peering_drop = proto::PeeringDrop {
            timestamp: self.timestamp as i64,
        };

        let mut bytes = BytesMut::with_capacity(peering_drop.encoded_len());

        // Panic: we have allocated a properly sized buffer.
        peering_drop
            .encode(&mut bytes)
            .expect("encoding drop-peering request failed");

        bytes
    }
}

impl fmt::Debug for DropPeeringRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DropPeeringRequest")
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("missing salt")]
    MissingSalt,
    #[error("invalid salt")]
    InvalidSalt,
    #[error("{0}")]
    ProtobufDecode(#[from] DecodeError),
    #[error("{0}")]
    ProtobufEncode(#[from] EncodeError),
    #[error("restore request hash")]
    RestoreRequestHash,
}
