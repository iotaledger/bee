// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types related to messages.

use crate::{consts::IOTA, proto};

use base64 as bs64;
use num_derive::FromPrimitive;
use prost::{bytes::BytesMut, DecodeError, EncodeError, Message as _};

use std::fmt;

pub(crate) struct Message(proto::Message);

/// The type of a message.
#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(u8)]
#[non_exhaustive]
pub enum MessageType {
    /// Send a message
    Message = 20 + IOTA,
    /// Request a message
    MessageRequest,
}

impl Message {
    pub(crate) fn new(data: &[u8]) -> Self {
        Self(proto::Message { data: data.to_vec() })
    }

    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(proto::Message::decode(bytes)?))
    }

    #[allow(dead_code)]
    pub(crate) fn data(&self) -> &Vec<u8> {
        &self.0.data
    }

    pub(crate) fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        let len = self.0.encoded_len();

        let mut buf = BytesMut::with_capacity(len);

        self.0.encode(&mut buf)?;

        Ok(buf)
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.0.data
    }
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Message")
            .field("data", &bs64::encode(&self.0.data))
            .finish()
    }
}

pub(crate) struct MessageRequest(proto::MessageRequest);

impl MessageRequest {
    pub(crate) fn new(id: &[u8]) -> Self {
        Self(proto::MessageRequest { id: id.to_vec() })
    }

    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(proto::MessageRequest::decode(bytes)?))
    }

    #[allow(dead_code)]
    pub(crate) fn data(&self) -> &Vec<u8> {
        &self.0.id
    }

    pub(crate) fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        let len = self.0.encoded_len();

        let mut buf = BytesMut::with_capacity(len);

        self.0.encode(&mut buf)?;

        Ok(buf)
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.0.id
    }
}

impl fmt::Debug for MessageRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessageRequest")
            .field("id", &bs64::encode(&self.0.id))
            .finish()
    }
}
