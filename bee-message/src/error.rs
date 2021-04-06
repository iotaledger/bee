// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    InvalidAmount(u64),
    InvalidDustAllowanceAmount(u64),
    InvalidTreasuryAmount(u64),
    InvalidMigratedFundsEntryAmount(u64),
    InvalidInputOutputCount(usize),
    InvalidUnlockBlockCount(usize),
    InvalidInputOutputIndex(u16),
    InvalidReferenceIndex(u16),
    InvalidInputKind(u8),
    InvalidOutputKind(u8),
    InvalidEssenceKind(u8),
    InvalidPayloadKind(u32),
    InvalidAddressKind(u8),
    InvalidSignatureKind(u8),
    InvalidUnlockBlockKind(u8),
    InvalidAccumulatedOutput(u128),
    InputUnlockBlockCountMismatch(usize, usize),
    InvalidParentsCount(usize),
    DuplicateError,
    InvalidAddress,
    MissingField(&'static str),
    InvalidPayloadLength(usize, usize),
    MissingPayload,
    InvalidHexadecimalChar(String),
    InvalidHexadecimalLength(usize, usize),
    InvalidIndexationIndexLength(usize),
    InvalidIndexationDataLength(usize),
    InvalidMessageLength(usize),
    InvalidReceiptFundsCount(usize),
    MilestonePublicKeysNotUniqueSorted,
    MilestoneInvalidPublicKeyCount(usize),
    MilestoneInvalidSignatureCount(usize),
    MilestonePublicKeysSignaturesCountMismatch(usize, usize),
    InvalidUnlockBlockReference(usize),
    DuplicateSignature(usize),
    TransactionInputsNotSorted,
    TransactionOutputsNotSorted,
    MigratedFundsNotSorted,
    RemainingBytesAfterMessage,
    ParentsNotUniqueSorted,
    TailTransactionHashNotUnique(usize, usize),
    SignaturePublicKeyMismatch(String, String),
    InvalidSignature,
    InvalidTailTransactionHash,
    InvalidPowScoreValues(u32, u32),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error happened: {}.", e),
            Error::InvalidAmount(amount) => write!(f, "Invalid amount: {}.", amount),
            Error::InvalidDustAllowanceAmount(amount) => write!(f, "Invalid dust allowance amount: {}.", amount),
            Error::InvalidTreasuryAmount(amount) => write!(f, "Invalid treasury amount: {}.", amount),
            Error::InvalidMigratedFundsEntryAmount(amount) => {
                write!(f, "Invalid migrated funds entry amount: {}.", amount)
            }
            Error::InvalidInputOutputCount(count) => write!(f, "Invalid input or output count: {}.", count),
            Error::InvalidUnlockBlockCount(count) => write!(f, "Invalid unlock block count: {}.", count),
            Error::InvalidInputOutputIndex(index) => write!(f, "Invalid input or output index: {}.", index),
            Error::InvalidReferenceIndex(index) => write!(f, "Invalid reference index: {}.", index),
            Error::InvalidInputKind(k) => write!(f, "Invalid input kind: {}.", k),
            Error::InvalidOutputKind(k) => write!(f, "Invalid output kind: {}.", k),
            Error::InvalidEssenceKind(k) => write!(f, "Invalid essence kind: {}.", k),
            Error::InvalidPayloadKind(k) => write!(f, "Invalid payload kind: {}.", k),
            Error::InvalidAddressKind(k) => write!(f, "Invalid address kind: {}.", k),
            Error::InvalidSignatureKind(k) => write!(f, "Invalid signature kind: {}.", k),
            Error::InvalidUnlockBlockKind(k) => write!(f, "Invalid unlock block kind: {}.", k),
            Error::InvalidAccumulatedOutput(value) => write!(f, "Invalid accumulated output balance: {}.", value),
            Error::InputUnlockBlockCountMismatch(input, block) => {
                write!(
                    f,
                    "Input count and unlock block count mismatch: {} != {}.",
                    input, block
                )
            }
            Error::InvalidParentsCount(count) => {
                write!(f, "Invalid parents count: {}.", count)
            }
            Error::DuplicateError => write!(f, "The object in the set must be unique."),
            Error::InvalidAddress => write!(f, "Invalid address provided."),
            Error::MissingField(s) => write!(f, "Missing required field: {}.", s),
            Error::InvalidPayloadLength(expected, actual) => {
                write!(f, "Invalid payload length: expected {}, got {}.", expected, actual)
            }
            Error::MissingPayload => write!(f, "Missing payload."),
            Error::InvalidHexadecimalChar(hex) => write!(f, "Invalid hexadecimal character: {}.", hex),
            Error::InvalidHexadecimalLength(expected, actual) => {
                write!(f, "Invalid hexadecimal length: expected {} got {}.", expected, actual)
            }
            Error::InvalidIndexationIndexLength(length) => {
                write!(f, "Invalid indexation index length {}.", length)
            }
            Error::InvalidIndexationDataLength(length) => {
                write!(f, "Invalid indexation data length {}.", length)
            }
            Error::InvalidMessageLength(length) => write!(f, "Invalid message length {}.", length),
            Error::InvalidReceiptFundsCount(count) => write!(f, "Invalid receipt funds count: {}.", count),
            Error::MilestonePublicKeysNotUniqueSorted => {
                write!(f, "Milestone public keys are not unique and/or sorted.")
            }
            Error::MilestoneInvalidPublicKeyCount(count) => {
                write!(f, "Invalid milestone public key count: {}.", count)
            }
            Error::MilestoneInvalidSignatureCount(count) => {
                write!(f, "Invalid milestone signature count: {}.", count)
            }
            Error::MilestonePublicKeysSignaturesCountMismatch(kcount, scount) => {
                write!(
                    f,
                    "Milestone public keys and signatures count mismatch: {0} != {1}.",
                    kcount, scount
                )
            }
            Error::InvalidUnlockBlockReference(index) => {
                write!(f, "Invalid unlock block reference: {0}", index)
            }
            Error::DuplicateSignature(index) => {
                write!(f, "Duplicate signature at index: {0}", index)
            }
            Error::TransactionInputsNotSorted => {
                write!(f, "Transaction inputs are not sorted.")
            }
            Error::TransactionOutputsNotSorted => {
                write!(f, "Transaction outputs are not sorted.")
            }
            Error::MigratedFundsNotSorted => {
                write!(f, "Migrated funds are not sorted.")
            }
            Error::RemainingBytesAfterMessage => {
                write!(f, "Remaining bytes after message.")
            }
            Error::ParentsNotUniqueSorted => {
                write!(f, "Parents not unique and/or sorted.")
            }
            Error::TailTransactionHashNotUnique(previous, current) => {
                write!(
                    f,
                    "Tail transaction hash is not unique at indices: {0} and {1}.",
                    previous, current
                )
            }
            Error::SignaturePublicKeyMismatch(expected, actual) => {
                write!(
                    f,
                    "Signature public key mismatch: expected {0}, got {1}.",
                    expected, actual
                )
            }
            Error::InvalidSignature => write!(f, "Invalid signature provided."),
            Error::InvalidTailTransactionHash => write!(f, "Invalid tail transaction hash."),
            Error::InvalidPowScoreValues(nps, npsmi) => write!(
                f,
                "Invalid pow score values: next pow score {} and next pow score milestone index {}.",
                nps, npsmi
            ),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}
