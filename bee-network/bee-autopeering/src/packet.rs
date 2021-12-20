// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! IOTA network packets.

use crate::{peer::peer_id::PeerId, proto};

use base64 as bs64;
use crypto::signatures::ed25519::{PublicKey, Signature};
use num_derive::FromPrimitive;
use prost::{bytes::BytesMut, DecodeError, EncodeError, Message};

use std::{fmt, net::SocketAddr, ops::Range};

// From `hive.go` docs:
// * specifies the maximum allowed size of packets;
// * packets larger than this will be cut and thus treated as invalid;
pub(crate) const MAX_PACKET_SIZE: usize = 1280;

pub(crate) const DISCOVERY_MSG_TYPE_MIN: u8 = 10;
pub(crate) const DISCOVERY_MSG_TYPE_RANGE: Range<u8> = DISCOVERY_MSG_TYPE_MIN..(DISCOVERY_MSG_TYPE_MIN + 4);
pub(crate) const PEERING_MSG_TYPE_MIN: u8 = 20;
pub(crate) const PEERING_MSG_TYPE_RANGE: Range<u8> = PEERING_MSG_TYPE_MIN..(PEERING_MSG_TYPE_MIN + 3);

/// Represents an IOTA packet.
pub(crate) struct Packet {
    msg_type: MessageType,
    msg_bytes: Vec<u8>,
    public_key: PublicKey,
    signature: Signature,
}

impl Packet {
    /// Creates a new packet.
    pub(crate) fn new(msg_type: MessageType, msg_bytes: &[u8], public_key: PublicKey, signature: Signature) -> Self {
        Self {
            msg_type,
            msg_bytes: msg_bytes.to_vec(),
            public_key,
            signature,
        }
    }

    /// Returns the message bytes contained in this packet.
    pub(crate) fn msg_bytes(&self) -> &[u8] {
        // &self.0.data
        &self.msg_bytes
    }

    /// Returns the public key belonging to the issuer of this packet.
    pub(crate) fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the signature belonging to the issuer of this packet.
    pub(crate) fn signature(&self) -> &Signature {
        &self.signature
    }

    /// Restores a packet from its protobuf representation.
    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, Error> {
        let proto::Packet {
            r#type,
            data,
            public_key,
            signature,
        } = proto::Packet::decode(bytes)?;

        let public_key = PublicKey::try_from_bytes(public_key.try_into().map_err(|_| Error::RestorePublicKey)?)
            .map_err(|_| Error::RestorePublicKey)?;

        let signature = Signature::from_bytes(signature.try_into().map_err(|_| Error::RestoreSignature)?);

        Ok(Self {
            msg_type: num::FromPrimitive::from_u8(r#type as u8).ok_or(Error::UnknownMessageType)?,
            msg_bytes: data.to_vec(),
            public_key,
            signature,
        })
    }

    /// Returns the protobuf representation of this packet
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_protobuf(&self) -> BytesMut {
        let proto_packet = proto::Packet {
            r#type: self.msg_type as u32,
            data: self.msg_bytes.to_vec(),
            public_key: self.public_key.to_bytes().to_vec(),
            signature: self.signature.to_bytes().to_vec(),
        };

        let mut buf = BytesMut::with_capacity(proto_packet.encoded_len());

        // Panic: we have allocated a properly sized buffer.
        proto_packet.encode(&mut buf).expect("encoding packet failed");

        buf
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Packet")
            .field("msg_type", &self.msg_type)
            .field("msg_bytes", &bs64::encode(&self.msg_bytes))
            .field("public_key", &bs58::encode(&self.public_key).into_string())
            .field("signature", &bs58::encode(&self.signature.to_bytes()).into_string())
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("prost decode error")]
    ProtobufDecode(#[from] DecodeError),
    #[error("prost encode error")]
    ProtobufEncode(#[from] EncodeError),
    #[error("failed to restore public key")]
    RestorePublicKey,
    #[error("failed to restore signature")]
    RestoreSignature,
    #[error("unknown message type")]
    UnknownMessageType,
}

/// The possible types of messages stored in a packet.
#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(u8)]
#[non_exhaustive]
pub(crate) enum MessageType {
    VerificationRequest = DISCOVERY_MSG_TYPE_MIN,
    VerificationResponse,
    DiscoveryRequest,
    DiscoveryResponse,
    PeeringRequest = PEERING_MSG_TYPE_MIN,
    PeeringResponse,
    DropRequest,
}

#[derive(Debug)]
pub(crate) struct IncomingPacket {
    pub(crate) msg_type: MessageType,
    pub(crate) msg_bytes: Vec<u8>,
    pub(crate) peer_addr: SocketAddr,
    pub(crate) peer_id: PeerId,
}

#[derive(Debug)]
pub(crate) struct OutgoingPacket {
    pub(crate) msg_type: MessageType,
    pub(crate) msg_bytes: Vec<u8>,
    pub(crate) peer_addr: SocketAddr,
}
