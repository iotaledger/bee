// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::fmt;

#[derive(Debug)]
pub enum Error {
    AmountError,
    CountError,
    NoInput,
    NoOutput,
    DuplicateError,
    // TODO add index
    InvalidIndex,
    InvalidAddress,
    InvalidSignature,
    OrderError,
    HashError,
    PathError,
    MissingField(&'static str),
    Io(std::io::Error),
    InvalidVariant,
    Utf8String(alloc::string::FromUtf8Error),
    InvalidType(u8, u8),
    InvalidAnnouncedLength(usize, usize),
    InvalidSyntax,
    InvalidHex,
    EmptyIndex,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AmountError => write!(f, "Invalid amount provided."),
            Error::CountError => write!(f, "Invalid count number provided."),
            Error::NoInput => write!(f, "No input provided."),
            Error::NoOutput => write!(f, "No output provided."),
            Error::DuplicateError => write!(f, "The object in the set must be unique."),
            Error::InvalidIndex => write!(f, "Invalid index provided."),
            Error::InvalidAddress => write!(f, "Invalid address provided."),
            Error::InvalidSignature => write!(f, "Invalid signature provided."),
            Error::OrderError => write!(f, "The vector is not sorted by lexicographical order."),
            Error::HashError => write!(f, "The format of provided hash is not correct."),
            Error::PathError => write!(f, "The format of provided BIP32 path is not correct."),
            Error::MissingField(s) => write!(f, "Missing required field: {}.", s),
            Error::Io(e) => write!(f, "I/O error happened: {}.", e),
            Error::InvalidVariant => write!(f, "Invalid variant read."),
            Error::Utf8String(e) => write!(f, "Invalid Utf8 string read: {}.", e),
            Error::InvalidType(expected, actual) => write!(f, "Invalid type read: {}, {}.", expected, actual),
            Error::InvalidAnnouncedLength(expected, actual) => {
                write!(f, "Invalid announced length: {}, {}.", expected, actual)
            }
            Error::InvalidSyntax => write!(f, "Syntax validation failed."),
            Error::InvalidHex => write!(f, "Invalid hexadecimal conversion."),
            Error::EmptyIndex => write!(f, "Empty index payload."),
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
