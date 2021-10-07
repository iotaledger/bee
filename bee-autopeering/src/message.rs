// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::SocketAddr;

// TODO: consider renaming to Request/Response

#[derive(Debug)]
pub(crate) struct OutgoingMessage {
    pub(crate) bytes: Vec<u8>,
    pub(crate) target: SocketAddr,
}

#[derive(Debug)]
pub(crate) struct IncomingMessage {
    pub(crate) bytes: Vec<u8>,
    pub(crate) source: SocketAddr,
}
