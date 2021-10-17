// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{peer::Peer, proto, request::Request, service_map::ServiceMap};

use prost::{bytes::BytesMut, DecodeError, EncodeError, Message as _};

use std::{
    fmt,
    net::{IpAddr, SocketAddr},
};

#[derive(Clone, Copy)]
pub(crate) struct VerificationRequest {
    pub(crate) version: u32,
    pub(crate) network_id: u32,
    pub(crate) timestamp: u64,
    pub(crate) source_addr: SocketAddr,
    pub(crate) target_addr: IpAddr,
}

impl VerificationRequest {
    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn network_id(&self) -> u32 {
        self.network_id
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
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

    pub fn protobuf(&self) -> Result<BytesMut, EncodeError> {
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
    pub(crate) request_hash: Vec<u8>,
    pub(crate) services: ServiceMap,
    pub(crate) target_addr: IpAddr,
}

impl VerificationResponse {
    pub fn new(request_hash: &[u8], services: ServiceMap, target_addr: IpAddr) -> Self {
        Self {
            request_hash: request_hash.to_vec(),
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

    pub(crate) fn target_addr(&self) -> IpAddr {
        self.target_addr
    }

    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
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

    pub fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        let pong = proto::Pong {
            req_hash: self.request_hash.clone(),
            services: Some(self.services.clone().into()),
            dst_addr: self.target_addr.to_string(),
        };

        let mut bytes = BytesMut::with_capacity(pong.encoded_len());
        pong.encode(&mut bytes)?;

        Ok(bytes)
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
    pub(crate) timestamp: u64,
}

impl DiscoveryRequest {
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        let proto::DiscoveryRequest { timestamp } = proto::DiscoveryRequest::decode(bytes)?;

        Ok(Self {
            timestamp: timestamp as u64,
        })
    }

    pub fn protobuf(&self) -> Result<BytesMut, EncodeError> {
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
    pub(crate) request_hash: Vec<u8>,
    pub(crate) peers: Vec<Peer>,
}

impl DiscoveryResponse {
    pub fn new(request_hash: Vec<u8>, peers: Vec<Peer>) -> Self {
        Self { request_hash, peers }
    }

    pub(crate) fn request_hash(&self) -> &[u8] {
        &self.request_hash
    }

    pub(crate) fn peers(&self) -> &[Peer] {
        &self.peers
    }

    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        let proto::DiscoveryResponse { req_hash, peers } = proto::DiscoveryResponse::decode(bytes)?;
        let peers = peers.into_iter().map(|peer| peer.into()).collect();

        Ok(Self {
            request_hash: req_hash,
            peers,
        })
    }

    pub fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        let peers = self.peers.clone().into_iter().map(|peer| peer.into()).collect();

        let disc_res = proto::DiscoveryResponse {
            req_hash: self.request_hash.clone(),
            peers,
        };

        let mut bytes = BytesMut::with_capacity(disc_res.encoded_len());
        disc_res.encode(&mut bytes)?;

        Ok(bytes)
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
