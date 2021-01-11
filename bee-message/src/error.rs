// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::fmt;

#[derive(Debug)]
pub enum Error {
    AmountError,
    InvalidInputOutputCount,
    InvalidInputType,
    InvalidOutputType,
    InvalidPayloadType,
    InvalidAddressType,
    InvalidSignatureType,
    InvalidUnlockType,
    NoInput,
    NoOutput,
    DuplicateError,
    // TODO add index
    InvalidIndex,
    InvalidAddress,
    InvalidSignature,
    OrderError,
    MissingField(&'static str),
    Io(std::io::Error),
    Utf8String(alloc::string::FromUtf8Error),
    InvalidType(u8, u8),
    InvalidAnnouncedLength(usize, usize),
    InvalidSyntax,
    InvalidHex,
    InvalidIndexLength(usize),
    InvalidMessageLength(usize),
    InvalidTransactionPayload,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AmountError => write!(f, "Invalid amount provided."),
            Error::InvalidInputOutputCount => write!(f, "Invalid count number provided."),
            Error::InvalidInputType => write!(f, "Invalid input type."),
            Error::InvalidOutputType => write!(f, "Invalid output type."),
            Error::InvalidPayloadType => write!(f, "Invalid payload type."),
            Error::InvalidAddressType => write!(f, "Invalid address type."),
            Error::InvalidSignatureType => write!(f, "Invalid signature type."),
            Error::InvalidUnlockType => write!(f, "Invalid unlock type."),
            Error::NoInput => write!(f, "No input provided."),
            Error::NoOutput => write!(f, "No output provided."),
            Error::DuplicateError => write!(f, "The object in the set must be unique."),
            Error::InvalidIndex => write!(f, "Invalid index provided."),
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
            Error::InvalidSyntax => write!(f, "Syntax validation failed."),
            Error::InvalidHex => write!(f, "Invalid hexadecimal conversion."),
            Error::InvalidIndexLength(length) => write!(f, "Invalid index length {}.", length),
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
