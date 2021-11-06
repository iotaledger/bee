// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{local::services::ServiceMap, peer::Peer, proto, request::Request};

use prost::{bytes::BytesMut, DecodeError, EncodeError, Message as _};

use std::{
    fmt,
    net::{IpAddr, SocketAddr},
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
    pub fn new(version: u32, network_id: u32, source_addr: SocketAddr, target_addr: IpAddr) -> Self {
        let timestamp = crate::time::unix_now_secs();

        Self {
            version,
            network_id,
            timestamp,
            source_addr,
            target_addr,
        }
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn network_id(&self) -> u32 {
        self.network_id
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn source_addr(&self) -> SocketAddr {
        self.source_addr
    }

    // TODO: revisit dead code
    #[allow(dead_code)]
    pub fn target_addr(&self) -> IpAddr {
        self.target_addr
    }

    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        let proto::Ping {
            version,
            network_id,
            timestamp,
            src_addr,
            src_port,
            dst_addr,
        } = proto::Ping::decode(bytes)?;

        let ip_addr: IpAddr = src_addr.parse().expect("error parsing ping source address");
        let port = src_port as u16;

        let source_addr = SocketAddr::new(ip_addr, port);
        let target_addr: IpAddr = dst_addr.parse().expect("error parsing ping target address");

        Ok(Self {
            version,
            network_id,
            timestamp: timestamp as u64,
            source_addr,
            target_addr,
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_protobuf(&self) -> Result<BytesMut, EncodeError> {
        let ping = proto::Ping {
            version: self.version,
            network_id: self.network_id,
            timestamp: self.timestamp as i64,
            src_addr: self.source_addr.ip().to_string(),
            src_port: self.source_addr.port() as u32,
            dst_addr: self.target_addr.to_string(),
        };

        let mut bytes = BytesMut::with_capacity(ping.encoded_len());
        ping.encode(&mut bytes)?;

        Ok(bytes)
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
    request_hash: Vec<u8>,
    services: ServiceMap,
    target_addr: IpAddr,
}

impl VerificationResponse {
    pub(crate) fn new(request_hash: Vec<u8>, services: ServiceMap, target_addr: IpAddr) -> Self {
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

    // TODO: revisit dead code
    /// When sent contains the external addr of the remote peer, when received the external addr of the local peer.
    #[allow(dead_code)]
    pub(crate) fn target_addr(&self) -> IpAddr {
        self.target_addr
    }

    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        let proto::Pong {
            req_hash,
            services,
            dst_addr,
        } = proto::Pong::decode(bytes)?;

        Ok(Self {
            request_hash: req_hash,
            services: services.expect("missing services").into(),
            target_addr: dst_addr.parse().expect("invalid target address"),
        })
    }

    pub(crate) fn to_protobuf(&self) -> Result<BytesMut, EncodeError> {
        let pong = proto::Pong {
            req_hash: self.request_hash.clone(),
            services: Some(self.services.clone().into()),
            dst_addr: self.target_addr.to_string(),
        };

        let mut bytes = BytesMut::with_capacity(pong.encoded_len());
        pong.encode(&mut bytes)?;

        Ok(bytes)
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
    pub(crate) fn to_protobuf(&self) -> Result<BytesMut, EncodeError> {
        let discover_request = proto::DiscoveryRequest {
            timestamp: self.timestamp as i64,
        };

        let mut bytes = BytesMut::with_capacity(discover_request.encoded_len());
        discover_request.encode(&mut bytes)?;

        Ok(bytes)
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
    request_hash: Vec<u8>,
    peers: Vec<Peer>,
}

impl DiscoveryResponse {
    pub(crate) fn new(request_hash: Vec<u8>, peers: Vec<Peer>) -> Self {
        Self { request_hash, peers }
    }

    pub(crate) fn request_hash(&self) -> &[u8] {
        &self.request_hash
    }

    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) fn peers(&self) -> &[Peer] {
        &self.peers
    }

    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        let proto::DiscoveryResponse { req_hash, peers } = proto::DiscoveryResponse::decode(bytes)?;
        let peers = peers.into_iter().map(proto::Peer::into).collect();

        Ok(Self {
            request_hash: req_hash,
            peers,
        })
    }

    pub(crate) fn to_protobuf(&self) -> Result<BytesMut, EncodeError> {
        let peers = self.peers.clone().into_iter().map(Peer::into).collect();

        let disc_res = proto::DiscoveryResponse {
            req_hash: self.request_hash.clone(),
            peers,
        };

        let mut bytes = BytesMut::with_capacity(disc_res.encoded_len());
        disc_res.encode(&mut bytes)?;

        Ok(bytes)
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