// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::ManualPeerConfig,
    conn::Direction,
    consts::{HANDSHAKE_TIMEOUT_SECS, MAX_HANDSHAKE_PACKET_SIZE, VERSION},
    identity::{Identity, LocalIdentity},
    packet::{packet_hash, Packet, PacketType},
    proto,
};

use crypto::signatures::ed25519;
use prost::{bytes::BytesMut, Message as _};

use std::{
    fmt,
    net::{IpAddr, SocketAddr},
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

type Alias = String;

pub async fn handshake(
    stream: TcpStream,
    socket_addr: SocketAddr,
    local_id: &LocalIdentity,
    direction: Direction,
    peer_config: ManualPeerConfig,
) -> Result<(BufReader<OwnedReadHalf>, BufWriter<OwnedWriteHalf>, Identity, Alias), HandshakeError> {
    log::info!("handshaking with {}...", socket_addr);

    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    let peer_id = match direction {
        Direction::Outbound => {
            let local_req_data = send_handshake_request(&mut writer, socket_addr.ip(), local_id).await?;
            await_response(&mut reader, local_req_data, &peer_config).await?
        }
        Direction::Inbound => await_request(&mut reader, &mut writer, local_id, &peer_config).await?,
    };

    Ok((reader, writer, peer_id, String::new()))
}

async fn send_handshake_request(
    writer: &mut BufWriter<OwnedWriteHalf>,
    to: IpAddr,
    local_id: &LocalIdentity,
) -> Result<BytesMut, HandshakeError> {
    let ty = PacketType::Handshake;

    let data = HandshakeRequest::new(to).protobuf()?;
    let signature = local_id.sign(&data).to_bytes();

    let packet = Packet::new(ty, &data, local_id.public_key().as_ref(), &signature);
    let packet_bytes = packet.protobuf().map_err(HandshakeError::Encode)?;

    writer
        .write_all(packet_bytes.as_ref())
        .await
        .map_err(HandshakeError::Io)?;
    writer.flush().await.map_err(HandshakeError::Io)?;

    Ok(data)
}

async fn send_handshake_response(
    writer: &mut BufWriter<OwnedWriteHalf>,
    req_data: &[u8],
    local_id: &LocalIdentity,
) -> Result<(), HandshakeError> {
    let ty = PacketType::Handshake;

    let data = HandshakeResponse::new(req_data).protobuf()?;
    let signature = local_id.sign(&data).to_bytes();

    let packet = Packet::new(ty, &data, local_id.public_key().as_ref(), &signature);
    let packet_bytes = packet.protobuf().map_err(HandshakeError::Encode)?;

    writer
        .write_all(packet_bytes.as_ref())
        .await
        .map_err(HandshakeError::Io)?;
    writer.flush().await.map_err(HandshakeError::Io)?;

    Ok(())
}

async fn await_request(
    reader: &mut BufReader<OwnedReadHalf>,
    writer: &mut BufWriter<OwnedWriteHalf>,
    local_id: &LocalIdentity,
    _peer_config: &ManualPeerConfig,
) -> Result<Identity, HandshakeError> {
    let mut buf = vec![0; MAX_HANDSHAKE_PACKET_SIZE];

    let packet = loop {
        if let Ok(num_received) = reader.read(&mut buf).await {
            if num_received == 0 {
                return Err(HandshakeError::ConnectionResetByPeer);
            }

            if num_received > MAX_HANDSHAKE_PACKET_SIZE {
                return Err(HandshakeError::PacketSizeMismatch {
                    received: num_received,
                    max_allowed: MAX_HANDSHAKE_PACKET_SIZE,
                });
            }

            let packet = Packet::from_protobuf(&buf[..num_received]).map_err(HandshakeError::Decode)?;
            let packet_type = packet.ty().map_err(HandshakeError::PacketType)?;

            if matches!(packet_type, PacketType::Handshake) {
                log::info!("received handshake request.");

                let req = HandshakeRequest::from_protobuf(packet.data())?;
                let peer_req_data = req.protobuf()?;

                HandshakeValidator::validate_request(&peer_req_data)?;

                send_handshake_response(writer, &peer_req_data, local_id).await?;

                break packet;
            }
        }
    };

    let peer_public_key = packet.public_key();
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&peer_public_key[..32]);
    let peer_public_key = ed25519::PublicKey::try_from_bytes(pk).map_err(HandshakeError::PublicKey)?;
    let peer_identity = Identity::from_public_key(peer_public_key);

    Ok(peer_identity)
}

async fn await_response(
    reader: &mut BufReader<OwnedReadHalf>,
    local_req_data: BytesMut,
    _peer_config: &ManualPeerConfig,
) -> Result<Identity, HandshakeError> {
    let mut buf = vec![0; MAX_HANDSHAKE_PACKET_SIZE];

    let packet = loop {
        if let Ok(num_received) = reader.read(&mut buf).await {
            if num_received == 0 {
                return Err(HandshakeError::ConnectionResetByPeer);
            }

            if num_received > MAX_HANDSHAKE_PACKET_SIZE {
                return Err(HandshakeError::PacketSizeMismatch {
                    received: num_received,
                    max_allowed: MAX_HANDSHAKE_PACKET_SIZE,
                });
            }

            let packet = Packet::from_protobuf(&buf[..num_received]).map_err(HandshakeError::Decode)?;
            let packet_type = packet.ty().map_err(HandshakeError::PacketType)?;

            if matches!(packet_type, PacketType::Handshake) {
                log::info!("received handshake response.");

                let res = HandshakeResponse::from_protobuf(packet.data())?;

                let peer_res_data = res.protobuf()?;

                HandshakeValidator::validate_response(&peer_res_data, &local_req_data)?;

                log::debug!("handshake response is valid");

                break packet;
            }
        }
    };

    let peer_public_key = packet.public_key();
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&peer_public_key[..32]);
    let peer_public_key = ed25519::PublicKey::try_from_bytes(pk).map_err(HandshakeError::PublicKey)?;
    let peer_identity = Identity::from_public_key(peer_public_key);

    Ok(peer_identity)
}

/// Errors, that may occur during the handshake process.
#[derive(Debug)]
pub enum HandshakeError {
    /// The peer dropped the connection.
    ConnectionResetByPeer,
    /// An I/O error occurred during handshaking.
    Io(io::Error),
    /// A protobuf encode error occurred.
    Encode(prost::EncodeError),
    /// A protobuf decode error occurred.
    Decode(prost::DecodeError),
    /// The peer doesn't use the same protocol version.
    VersionMismatch {
        /// The expected version.
        expected: u32,
        /// The received version.
        received: u32,
    },
    /// The handshake timestamp was too old.
    Expired,
    /// The peer didn't provide the expected hash in its handshake response.
    RequestHashMismatch,
    /// The handshake response came in too late.
    _ResponseTimeout,
    /// Wrong packet type.
    PacketType(io::Error),
    /// The peer send a packet which is too large.
    PacketSizeMismatch {
        /// The received packet size.
        received: usize,
        /// The maximum allowed packet size.
        max_allowed: usize,
    },
    /// TODO
    PublicKey(crypto::Error),
}

/// Represents a handshake request, and is a wrapper of the corresponding Protobuf type.
pub struct HandshakeRequest {
    inner: proto::HandshakeRequest,
}

impl HandshakeRequest {
    /// Creates a new handshake request.
    pub fn new(to: IpAddr) -> Self {
        let inner = proto::HandshakeRequest {
            version: VERSION,
            to: to.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Wake up, NEO. Follow the White Rabbit.")
                .as_secs() as i64,
        };

        Self { inner }
    }

    /// Creates a new instance from its protobuf encoded byte representation.
    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, HandshakeError> {
        let inner = proto::HandshakeRequest::decode(bytes).map_err(HandshakeError::Decode)?;

        Ok(Self { inner })
    }

    /// Returns the protocol version.
    pub fn version(&self) -> u32 {
        self.inner.version
    }

    /// Returns the handshaking peer's address.
    #[allow(dead_code)]
    pub fn to_addr(&self) -> &String {
        &self.inner.to
    }

    /// Returns the creation timestamp of this handshake request.
    pub fn timestamp(&self) -> i64 {
        self.inner.timestamp
    }

    /// Returns the protobuf byte representation of this handshake request.
    pub fn protobuf(&self) -> Result<BytesMut, HandshakeError> {
        let len = self.inner.encoded_len();

        let mut bytes = BytesMut::with_capacity(len);

        self.inner.encode(&mut bytes).map_err(HandshakeError::Encode)?;

        Ok(bytes)
    }
}

impl fmt::Debug for HandshakeRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandshakeRequest")
            .field("version", &self.inner.version)
            .field("to", &self.inner.to)
            .field("timestamp", &self.inner.timestamp)
            .finish()
    }
}

/// Represents a handshake response, and is a wrapper of the corresponding Protobuf type.
pub struct HandshakeResponse {
    inner: proto::HandshakeResponse,
}

impl HandshakeResponse {
    /// Creates a new handshake response from the request of the peer.
    pub fn new(req_data: &[u8]) -> Self {
        let inner = proto::HandshakeResponse {
            req_hash: packet_hash(req_data),
        };
        Self { inner }
    }

    /// Creates a new instance from its protobuf encoded byte representation.
    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, HandshakeError> {
        let inner = proto::HandshakeResponse::decode(bytes).map_err(HandshakeError::Decode)?;

        Ok(Self { inner })
    }

    /// Returns the hash of the associated request that was issued by the peer.
    pub fn req_hash(&self) -> &Vec<u8> {
        &self.inner.req_hash
    }

    /// Returns the protobuf byte representation of this handshake response.
    pub fn protobuf(&self) -> Result<BytesMut, HandshakeError> {
        let len = self.inner.encoded_len();

        let mut bytes = BytesMut::with_capacity(len);

        self.inner.encode(&mut bytes).map_err(HandshakeError::Encode)?;

        Ok(bytes)
    }
}

impl fmt::Debug for HandshakeResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandshakeRequest")
            .field("req_hash", &bs58::encode(&self.inner.req_hash).into_string())
            .finish()
    }
}

pub struct HandshakeValidator;

impl HandshakeValidator {
    pub fn validate_request(req_data: &[u8]) -> Result<(), HandshakeError> {
        let req = HandshakeRequest::from_protobuf(req_data)?;

        if req.version() != VERSION {
            return Err(HandshakeError::VersionMismatch {
                expected: VERSION,
                received: req.version(),
            });
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock went backwards")
            .as_secs() as i64;

        if timestamp - req.timestamp() > HANDSHAKE_TIMEOUT_SECS as i64 {
            return Err(HandshakeError::Expired);
        }

        Ok(())
    }

    pub fn validate_response(res_data: &[u8], req_data: &[u8]) -> Result<(), HandshakeError> {
        let res = HandshakeResponse::from_protobuf(res_data)?;

        let expected_req_hash = &packet_hash(req_data);
        let received_req_hash = res.req_hash();
        if received_req_hash != expected_req_hash {
            return Err(HandshakeError::RequestHashMismatch);
        }

        Ok(())
    }
}
