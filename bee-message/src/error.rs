// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidAmount(u64),
    InvalidInputOutputCount(usize),
    InvalidInputOutputIndex(u16),
    InvalidInputType(u8),
    InvalidOutputType(u8),
    InvalidPayloadType(u32),
    InvalidAddressType(u8),
    InvalidSignatureType(u8),
    InvalidUnlockType(u8),
    InvalidAccumulatedOutput(u128),
    NoInput,
    NoOutput,
    DuplicateError,
    InvalidAddress,
    InvalidSignature,
    OrderError,
    MissingField(&'static str),
    Io(std::io::Error),
    Utf8String(alloc::string::FromUtf8Error),
    InvalidType(u8, u8),
    InvalidAnnouncedLength(usize, usize),
    InvalidHex,
    InvalidIndexationLength(usize),
    InvalidMessageLength(usize),
    InvalidTransactionPayload,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidAmount(amount) => write!(f, "Invalid amount: {}.", amount),
            Error::InvalidInputOutputCount(count) => write!(f, "Invalid input or output count: {}.", count),
            Error::InvalidInputOutputIndex(index) => write!(f, "Invalid input or output index: {}.", index),
            Error::InvalidInputType(t) => write!(f, "Invalid input type: {}.", t),
            Error::InvalidOutputType(t) => write!(f, "Invalid output type: {}.", t),
            Error::InvalidPayloadType(t) => write!(f, "Invalid payload type: {}.", t),
            Error::InvalidAddressType(t) => write!(f, "Invalid address type: {}.", t),
            Error::InvalidSignatureType(t) => write!(f, "Invalid signature type: {}.", t),
            Error::InvalidUnlockType(t) => write!(f, "Invalid unlock type: {}.", t),
            Error::InvalidAccumulatedOutput(value) => write!(f, "Invalid accumulated output balance: {}.", value),
            Error::NoInput => write!(f, "No input provided."),
            Error::NoOutput => write!(f, "No output provided."),
            Error::DuplicateError => write!(f, "The object in the set must be unique."),
            Error::InvalidAddress => write!(f, "Invalid address provided."),
            Error::InvalidSignature => write!(f, "Invalid signature provided."),
            Error::OrderError => write!(f, "The vector is not sorted by lexicographical order."),
            Error::MissingField(s) => write!(f, "Missing required field: {}.", s),
            Error::Io(e) => write!(f, "I/O error happened: {}.", e),
            Error::Utf8String(e) => write!(f, "Invalid Utf8 string read: {}.", e),
            Error::InvalidType(expected, actual) => write!(f, "Invalid type read: {}, {}.", expected, actual),
            Error::InvalidAnnouncedLength(expected, actual) => {
                write!(f, "Invalid announced length: {}, {}.", expected, actual)
            }
            Error::InvalidHex => write!(f, "Invalid hexadecimal conversion."),
            Error::InvalidIndexationLength(length) => write!(f, "Invalid indexation index or data length {}.", length),
            Error::InvalidMessageLength(length) => write!(f, "Invalid message length {}.", length),
            Error::InvalidTransactionPayload => write!(f, "Invalid transaction payload type."),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<alloc::string::FromUtf8Error> for Error {
    fn from(error: alloc::string::FromUtf8Error) -> Self {
        Error::Utf8String(error)
    }
}
