// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! IOTA network packets.

use crate::proto;

use base64 as bs64;
use num_derive::FromPrimitive;
use prost::{bytes::BytesMut, DecodeError, EncodeError, Message};

use std::{fmt, io, net::SocketAddr};

// From `hive.go` docs:
// * specifies the maximum allowed size of packets;
// * packets larger than this will be cut and thus treated as invalid;
const MAX_PACKET_SIZE: usize = 1280;
const PACKET_TYPE_MIN: u32 = 20;

/// Represents an IOTA packet.
pub struct Packet(proto::Packet);

impl Packet {
    /// Creates a new packet.
    pub fn new(packet_type: PacketType, msg: &[u8], public_key: &[u8], signature: &[u8]) -> Self {
        Self(proto::Packet {
            r#type: packet_type as u32,
            data: msg.to_vec(),
            public_key: public_key.to_vec(),
            signature: signature.to_vec(),
        })
    }

    /// Returns the type of this packet.
    pub fn packet_type(&self) -> Result<PacketType, io::Error> {
        num::FromPrimitive::from_u32(self.0.r#type)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "unknown packet type identifier"))
    }

    /// Returns the message contained in this packet.
    pub fn message(&self) -> &Vec<u8> {
        &self.0.data
    }

    /// Returns the public key belonging to the issuer of this packet.
    pub fn public_key(&self) -> &Vec<u8> {
        &self.0.public_key
    }

    /// Returns the signature belonging to the issuer of this packet.
    #[allow(dead_code)]
    pub fn signature(&self) -> &Vec<u8> {
        &self.0.signature
    }

    /// Restores a packet from its protobuf representation.
    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(proto::Packet::decode(bytes)?))
    }

    /// Returns the protobuf representation of this packet
    pub fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        let mut buf = BytesMut::with_capacity(self.0.encoded_len());
        self.0.encode(&mut buf)?;

        Ok(buf)
    }

    /// Turns the packet into its contained message (if any) and discards the rest of the metadata.
    pub fn into_message(self) -> Vec<u8> {
        self.0.data
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Packet")
            .field("msg_type", &self.0.r#type)
            .field("msg", &bs64::encode(&self.0.data))
            .field("public_key", &bs58::encode(&self.0.public_key).into_string())
            .field("signature", &bs58::encode(&self.0.signature).into_string())
            .finish()
    }
}

/// The possible types of packets.
#[derive(Debug, FromPrimitive)]
#[repr(u32)]
#[non_exhaustive]
pub enum PacketType {
    PeeringRequest = PACKET_TYPE_MIN,
    PeeringResponse,
    PeeringDrop,
}

#[derive(Debug)]
pub(crate) struct IncomingPacket {
    pub(crate) bytes: Vec<u8>,
    pub(crate) source: SocketAddr,
}

#[derive(Debug)]
pub(crate) struct OutgoingPacket {
    pub(crate) bytes: Vec<u8>,
    pub(crate) target: SocketAddr,
}
