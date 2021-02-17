// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidAmount(u64),
    InvalidDustAllowanceAmount(u64),
    InvalidTreasuryAmount(u64),
    InvalidMigratedFundsEntryAmount(u64),
    InvalidInputOutputCount(usize),
    InvalidInputOutputIndex(u16),
    InvalidInputKind(u8),
    InvalidOutputKind(u8),
    InvalidEssenceKind(u8),
    InvalidPayloadKind(u32),
    InvalidAddressKind(u8),
    InvalidSignatureKind(u8),
    InvalidUnlockKind(u8),
    InvalidAccumulatedOutput(u128),
    InvalidUnlockBlockCount(usize, usize),
    InvalidParentsCount(usize),
    NoInput,
    NoOutput,
    DuplicateError,
    InvalidAddress,
    InvalidSignature,
    OrderError,
    MissingField(&'static str),
    Io(std::io::Error),
    Utf8String(alloc::string::FromUtf8Error),
    InvalidAnnouncedLength(usize, usize),
    InvalidHexadecimalChar(String),
    InvalidHexadecimalLength(usize, usize),
    InvalidIndexationLength(usize),
    InvalidMessageLength(usize),
    InvalidTransactionPayload,
    InvalidReceiptFundsCount(usize),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidAmount(amount) => write!(f, "Invalid amount: {}.", amount),
            Error::InvalidDustAllowanceAmount(amount) => write!(f, "Invalid dust allowance amount: {}.", amount),
            Error::InvalidTreasuryAmount(amount) => write!(f, "Invalid treasury amount: {}.", amount),
            Error::InvalidMigratedFundsEntryAmount(amount) => {
                write!(f, "Invalid migrated funds entry amount: {}.", amount)
            }
            Error::InvalidInputOutputCount(count) => write!(f, "Invalid input or output count: {}.", count),
            Error::InvalidInputOutputIndex(index) => write!(f, "Invalid input or output index: {}.", index),
            Error::InvalidInputKind(k) => write!(f, "Invalid input kind: {}.", k),
            Error::InvalidOutputKind(k) => write!(f, "Invalid output kind: {}.", k),
            Error::InvalidEssenceKind(k) => write!(f, "Invalid essence kind: {}.", k),
            Error::InvalidPayloadKind(k) => write!(f, "Invalid payload kind: {}.", k),
            Error::InvalidAddressKind(k) => write!(f, "Invalid address kind: {}.", k),
            Error::InvalidSignatureKind(k) => write!(f, "Invalid signature kind: {}.", k),
            Error::InvalidUnlockKind(k) => write!(f, "Invalid unlock kind: {}.", k),
            Error::InvalidAccumulatedOutput(value) => write!(f, "Invalid accumulated output balance: {}.", value),
            Error::InvalidUnlockBlockCount(input, block) => {
                write!(f, "Invalid unlock block count: {} != {}.", input, block)
            }
            Error::InvalidParentsCount(count) => {
                write!(f, "Invalid parents count: {}.", count)
            }
            Error::NoInput => write!(f, "No input provided."),
            Error::NoOutput => write!(f, "No output provided."),
            Error::DuplicateError => write!(f, "The object in the set must be unique."),
            Error::InvalidAddress => write!(f, "Invalid address provided."),
            Error::InvalidSignature => write!(f, "Invalid signature provided."),
            Error::OrderError => write!(f, "The vector is not sorted by lexicographical order."),
            Error::MissingField(s) => write!(f, "Missing required field: {}.", s),
            Error::Io(e) => write!(f, "I/O error happened: {}.", e),
            Error::Utf8String(e) => write!(f, "Invalid Utf8 string read: {}.", e),
            Error::InvalidAnnouncedLength(expected, actual) => {
                write!(f, "Invalid announced length: {}, {}.", expected, actual)
            }
            Error::InvalidHexadecimalChar(hex) => write!(f, "Invalid hexadecimal character: {}.", hex),
            Error::InvalidHexadecimalLength(expected, actual) => {
                write!(f, "Invalid hexadecimal length: expected {} got {}.", expected, actual)
            }
            Error::InvalidIndexationLength(length) => write!(f, "Invalid indexation index or data length {}.", length),
            Error::InvalidMessageLength(length) => write!(f, "Invalid message length {}.", length),
            Error::InvalidTransactionPayload => write!(f, "Invalid transaction payload kind."),
            Error::InvalidReceiptFundsCount(count) => write!(f, "Invalid receipt funds count: {}.", count),
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
