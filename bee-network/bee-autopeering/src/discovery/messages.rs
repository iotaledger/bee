// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{local::services::ServiceMap, peer::Peer, proto, request::Request};

use crypto::hashes::sha::SHA256_LEN;
use prost::{bytes::BytesMut, DecodeError, EncodeError, Message as _};

use std::{
    fmt,
    net::{AddrParseError, IpAddr, SocketAddr},
};

#[derive(Clone, Copy)]
pub(crate) struct VerificationRequest {
    version: u32,
    network_id: u32,
    timestamp: u64,
    source_addr: SocketAddr,
    target_addr: IpAddr,
}

impl VerificationRequest {
    pub(crate) fn new(version: u32, network_id: u32, source_addr: SocketAddr, target_addr: IpAddr) -> Self {
        let timestamp = crate::time::unix_now_secs();

        Self {
            version,
            network_id,
            timestamp,
            source_addr,
            target_addr,
        }
    }

    pub(crate) fn version(&self) -> u32 {
        self.version
    }

    pub(crate) fn network_id(&self) -> u32 {
        self.network_id
    }

    pub(crate) fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub(crate) fn source_addr(&self) -> SocketAddr {
        self.source_addr
    }

    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, Error> {
        let proto::Ping {
            version,
            network_id,
            timestamp,
            src_addr,
            src_port,
            dst_addr,
        } = proto::Ping::decode(bytes)?;

        let ip_addr: IpAddr = src_addr.parse().map_err(Error::InvalidSourceIpAddress)?;
        let port = src_port as u16;

        let source_addr = SocketAddr::new(ip_addr, port);
        let target_addr: IpAddr = dst_addr.parse().map_err(Error::InvalidTargetIpAddress)?;

        Ok(Self {
            version,
            network_id,
            timestamp: timestamp as u64,
            source_addr,
            target_addr,
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_protobuf(&self) -> BytesMut {
        let ping = proto::Ping {
            version: self.version,
            network_id: self.network_id,
            timestamp: self.timestamp as i64,
            src_addr: self.source_addr.ip().to_string(),
            src_port: self.source_addr.port() as u32,
            dst_addr: self.target_addr.to_string(),
        };

        let mut bytes = BytesMut::with_capacity(ping.encoded_len());

        // Panic: we have allocated a properly sized buffer.
        ping.encode(&mut bytes).expect("encoding discovery request failed");

        bytes
    }
}

impl fmt::Debug for VerificationRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VerificationRequest")
            .field("version", &self.version)
            .field("network_id", &self.network_id)
            .field("timestamp", &self.timestamp)
            .field("source_addr", &self.source_addr)
            .field("target_addr", &self.target_addr)
            .finish()
    }
}

impl Request for VerificationRequest {}

#[derive(Clone)]
pub(crate) struct VerificationResponse {
    request_hash: [u8; SHA256_LEN],
    services: ServiceMap,
    target_addr: IpAddr,
}

impl VerificationResponse {
    pub(crate) fn new(request_hash: [u8; SHA256_LEN], services: ServiceMap, target_addr: IpAddr) -> Self {
        Self {
            request_hash,
            services,
            target_addr,
        }
    }

    pub(crate) fn request_hash(&self) -> &[u8] {
        &self.request_hash
    }

    pub(crate) fn services(&self) -> &ServiceMap {
        &self.services
    }

    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, Error> {
        let proto::Pong {
            req_hash,
            services,
            dst_addr,
        } = proto::Pong::decode(bytes)?;

        Ok(Self {
            request_hash: req_hash.try_into().map_err(|_| Error::RestoreRequestHash)?,
            services: services.ok_or(Error::MissingServices)?.try_into()?,
            target_addr: dst_addr.parse().map_err(Error::InvalidTargetIpAddress)?,
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_protobuf(&self) -> BytesMut {
        let pong = proto::Pong {
            req_hash: self.request_hash.to_vec(),
            services: Some(self.services().into()),
            dst_addr: self.target_addr.to_string(),
        };

        let mut bytes = BytesMut::with_capacity(pong.encoded_len());

        // Panic: we have allocated a properly sized buffer.
        pong.encode(&mut bytes).expect("encoding discovery response failed");

        bytes
    }

    pub(crate) fn into_services(self) -> ServiceMap {
        self.services
    }
}

impl fmt::Debug for VerificationResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VerificationResponse")
            .field("request_hash", &bs58::encode(&self.request_hash).into_string())
            .field("services", &self.services.to_string())
            .field("target_addr", &self.target_addr.to_string())
            .finish()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct DiscoveryRequest {
    timestamp: u64,
}

impl DiscoveryRequest {
    pub(crate) fn new() -> Self {
        let timestamp = crate::time::unix_now_secs();

        Self { timestamp }
    }

    pub(crate) fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        let proto::DiscoveryRequest { timestamp } = proto::DiscoveryRequest::decode(bytes)?;

        Ok(Self {
            timestamp: timestamp as u64,
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_protobuf(&self) -> BytesMut {
        let discover_request = proto::DiscoveryRequest {
            timestamp: self.timestamp as i64,
        };

        let mut bytes = BytesMut::with_capacity(discover_request.encoded_len());

        // Panic: we have allocated a properly sized buffer.
        discover_request
            .encode(&mut bytes)
            .expect("encoding discovery request failed");

        bytes
    }
}

impl fmt::Debug for DiscoveryRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiscoveryRequest")
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

impl Request for DiscoveryRequest {}

#[derive(Clone)]
pub(crate) struct DiscoveryResponse {
    request_hash: [u8; SHA256_LEN],
    peers: Vec<Peer>,
}

impl DiscoveryResponse {
    pub(crate) fn new(request_hash: [u8; SHA256_LEN], peers: Vec<Peer>) -> Self {
        Self { request_hash, peers }
    }

    pub(crate) fn request_hash(&self) -> &[u8] {
        &self.request_hash
    }

    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, Error> {
        let proto::DiscoveryResponse { req_hash, peers } = proto::DiscoveryResponse::decode(bytes)?;
        let peers = peers
            .into_iter()
            .filter_map(|p| proto::Peer::try_into(p).ok())
            .collect();

        Ok(Self {
            request_hash: req_hash.try_into().expect("todo: error type"),
            peers,
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_protobuf(&self) -> BytesMut {
        let peers = self.peers.iter().map(Into::into).collect();

        let disc_res = proto::DiscoveryResponse {
            req_hash: self.request_hash.to_vec(),
            peers,
        };

        let mut bytes = BytesMut::with_capacity(disc_res.encoded_len());

        // Panic: we have allocated a properly sized buffer.
        disc_res.encode(&mut bytes).expect("encoding discovery response failed");

        bytes
    }

    pub(crate) fn into_peers(self) -> Vec<Peer> {
        self.peers
    }
}

impl fmt::Debug for DiscoveryResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiscoveryResponse")
            .field("request_hash", &bs58::encode(&self.request_hash).into_string())
            .field("peers", &self.peers)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("the peer did not announce any services")]
    MissingServices,
    #[error("invalid source ip address due to: {0}.")]
    InvalidSourceIpAddress(AddrParseError),
    #[error("invalid target ip address due to: {0}.")]
    InvalidTargetIpAddress(AddrParseError),
    #[error("invalid service description")]
    Service(#[from] crate::local::services::Error),
    #[error("{0}")]
    ProtobufDecode(#[from] DecodeError),
    #[error("{0}")]
    ProtobufEncode(#[from] EncodeError),
    #[error("restore request hash")]
    RestoreRequestHash,
}
