// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! IOTA network packets.

use crate::proto;

use base64 as bs64;
use crypto::hashes::sha;
use num_derive::FromPrimitive;
use prost::{bytes::BytesMut, DecodeError, EncodeError, Message};

use std::{fmt, io};

/// Returns the hash of the given packet bytes in its byte representation.
pub fn packet_hash(packet_bytes: &[u8]) -> Vec<u8> {
    let mut digest = [0; sha::SHA256_LEN];
    sha::SHA256(packet_bytes, &mut digest);
    digest.to_vec()
}

/// Represents an IOTA packet.
pub struct Packet(proto::Packet);

impl Packet {
    /// Creates a new packet.
    pub fn new(ty: PacketType, data: &[u8], public_key: &[u8], signature: &[u8]) -> Self {
        Self(proto::Packet {
            r#type: ty as u32,
            data: data.to_vec(),
            public_key: public_key.to_vec(),
            signature: signature.to_vec(),
        })
    }

    /// Restores a packet from its protobuf representation.
    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(proto::Packet::decode(bytes)?))
    }

    /// Returns the type of this packet.
    pub fn ty(&self) -> Result<PacketType, io::Error> {
        num::FromPrimitive::from_u32(self.0.r#type)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "unknown packet type identifier"))
    }

    /// Returns the data part of this packet.
    pub fn data(&self) -> &Vec<u8> {
        &self.0.data
    }

    /// Returns the public key of the issuer of this packet.
    pub fn public_key(&self) -> &Vec<u8> {
        &self.0.public_key
    }

    /// Returns the signature of the issuer of this packet.
    #[allow(dead_code)]
    pub fn signature(&self) -> &Vec<u8> {
        &self.0.signature
    }

    /// Returns the protobuf representation of this packet
    pub fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        let len = self.0.encoded_len();

        let mut buf = BytesMut::with_capacity(len);

        self.0.encode(&mut buf)?;

        Ok(buf)
    }

    /// Reduces the packet to its data part.
    pub fn into_data(self) -> Vec<u8> {
        self.0.data
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Packet")
            .field("type", &self.0.r#type)
            .field("data", &bs64::encode(&self.0.data))
            .field("public_key", &bs58::encode(&self.0.public_key).into_string())
            .field("signature", &bs58::encode(&self.0.signature).into_string())
            .finish()
    }
}

/// The possible types of packets.
#[derive(Debug, FromPrimitive)]
#[repr(u64)]
#[non_exhaustive]
pub enum PacketType {
    /// Handshake packet to establish a certain protocol with a peer.
    Handshake = 0,
}
