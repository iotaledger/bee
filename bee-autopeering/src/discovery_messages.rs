// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    hash, proto,
    service_map::{ServiceMap, ServiceName},
};

use prost::{bytes::BytesMut, DecodeError, EncodeError, Message as _};

use std::{
    fmt,
    net::{IpAddr, SocketAddr},
};

pub(crate) struct PingFactory {
    version: u32,
    network_id: u32,
    source_addr: SocketAddr,
}

impl PingFactory {
    pub fn new(version: u32, network_id: u32, source_addr: SocketAddr) -> Self {
        Self {
            version,
            network_id,
            source_addr,
        }
    }

    pub(crate) fn make(&self, target: IpAddr) -> Ping {
        let timestamp = crate::time::unix_now();

        Ping {
            version: self.version,
            network_id: self.network_id,
            timestamp,
            source_addr: self.source_addr,
            target_addr: target,
        }
    }
}
pub(crate) struct Ping {
    version: u32,
    network_id: u32,
    timestamp: u64,
    source_addr: SocketAddr,
    target_addr: IpAddr,
}

impl Ping {
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

        // Ok(Self {})
        todo!()
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

impl fmt::Debug for Ping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ping")
            .field("version", &self.version)
            .field("network_id", &self.network_id)
            .field("timestamp", &self.timestamp)
            .field("source_addr", &self.source_addr)
            .field("target_addr", &self.target_addr)
            .finish()
    }
}

pub(crate) struct Pong {
    ping_hash: Vec<u8>,
    services: ServiceMap,
    target_addr: IpAddr,
}

impl Pong {
    pub fn new(&self, ping_hash: Vec<u8>, services: ServiceMap, target_addr: IpAddr) -> Pong {
        Pong {
            ping_hash,
            services,
            target_addr,
        }
    }

    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        let proto::Pong {
            req_hash,
            services,
            dst_addr,
        } = proto::Pong::decode(bytes)?;

        Ok(Self {
            ping_hash: req_hash,
            services: services.expect("missing services").into(),
            target_addr: dst_addr.parse().expect("invalid target address"),
        })
    }

    pub fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        let pong = proto::Pong {
            req_hash: self.ping_hash.clone(),
            services: Some(self.services.clone().into()),
            dst_addr: self.target_addr.to_string(),
        };

        let mut bytes = BytesMut::with_capacity(pong.encoded_len());
        pong.encode(&mut bytes)?;

        Ok(bytes)
    }
}

impl fmt::Debug for Pong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pong")
            .field("ping_hash", &bs58::encode(&self.ping_hash).into_string())
            .field("services", &self.services.to_string())
            .field("target_addr", &self.target_addr.to_string())
            .finish()
    }
}
