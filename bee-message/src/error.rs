// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::input::UtxoInput;

use crypto::Error as CryptoError;

use core::fmt;

/// Error occurring when creating/parsing/validating messages.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Error {
    CryptoError(CryptoError),
    DuplicateSignature(u16),
    DuplicateUtxo(UtxoInput),
    InputUnlockBlockCountMismatch(usize, usize),
    InvalidAccumulatedOutput(u128),
    InvalidAddress,
    InvalidAddressKind(u8),
    InvalidAmount(u64),
    InvalidEssenceKind(u8),
    InvalidFeatureBlockKind(u8),
    InvalidHexadecimalChar(String),
    InvalidHexadecimalLength(usize, usize),
    InvalidIndexationDataLength(usize),
    InvalidIndexationIndexLength(usize),
    InvalidInputKind(u8),
    InvalidInputOutputCount(u16),
    InvalidInputOutputIndex(u16),
    InvalidMessageLength(usize),
    InvalidMigratedFundsEntryAmount(u64),
    InvalidOutputKind(u8),
    InvalidParentsCount(usize),
    InvalidPayloadKind(u32),
    InvalidPayloadLength(usize, usize),
    InvalidPowScoreValues(u32, u32),
    InvalidReceiptFundsCount(u16),
    InvalidReferenceIndex(u16),
    InvalidSignature,
    InvalidSignatureKind(u8),
    InvalidTailTransactionHash,
    InvalidTokenSchemeKind(u8),
    InvalidTreasuryAmount(u64),
    InvalidUnlockBlockCount(u16),
    InvalidUnlockBlockKind(u8),
    InvalidUnlockBlockReference(u16),
    InvalidUnlockBlockAlias(u16),
    InvalidUnlockBlockNft(u16),
    Io(std::io::Error),
    MigratedFundsNotSorted,
    MilestoneInvalidPublicKeyCount(usize),
    MilestoneInvalidSignatureCount(usize),
    MilestonePublicKeysNotUniqueSorted,
    MilestonePublicKeysSignaturesCountMismatch(usize, usize),
    MissingField(&'static str),
    MissingPayload,
    ParentsNotUniqueSorted,
    ReceiptFundsNotUniqueSorted,
    RemainingBytesAfterMessage,
    SignaturePublicKeyMismatch(String, String),
    TailTransactionHashNotUnique(usize, usize),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CryptoError(e) => write!(f, "Cryptographic error: {}.", e),
            Error::DuplicateUtxo(utxo) => write!(f, "Duplicate UTXO {:?} in inputs.", utxo),
            Error::DuplicateSignature(index) => {
                write!(f, "Duplicate signature at index: {0}", index)
            }
            Error::InputUnlockBlockCountMismatch(input, block) => {
                write!(
                    f,
                    "Input count and unlock block count mismatch: {} != {}.",
                    input, block
                )
            }
            Error::InvalidAccumulatedOutput(value) => write!(f, "Invalid accumulated output balance: {}.", value),
            Error::InvalidAddress => write!(f, "Invalid address provided."),
            Error::InvalidAddressKind(k) => write!(f, "Invalid address kind: {}.", k),
            Error::InvalidAmount(amount) => write!(f, "Invalid amount: {}.", amount),
            Error::InvalidEssenceKind(k) => write!(f, "Invalid essence kind: {}.", k),
            Error::InvalidFeatureBlockKind(k) => write!(f, "Invalid feature block kind: {}.", k),
            Error::InvalidHexadecimalChar(hex) => write!(f, "Invalid hexadecimal character: {}.", hex),
            Error::InvalidHexadecimalLength(expected, actual) => {
                write!(f, "Invalid hexadecimal length: expected {} got {}.", expected, actual)
            }
            Error::InvalidIndexationDataLength(length) => {
                write!(f, "Invalid indexation data length {}.", length)
            }
            Error::InvalidIndexationIndexLength(length) => {
                write!(f, "Invalid indexation index length {}.", length)
            }
            Error::InvalidInputKind(k) => write!(f, "Invalid input kind: {}.", k),
            Error::InvalidInputOutputCount(count) => write!(f, "Invalid input or output count: {}.", count),
            Error::InvalidInputOutputIndex(index) => write!(f, "Invalid input or output index: {}.", index),
            Error::InvalidMessageLength(length) => write!(f, "Invalid message length {}.", length),
            Error::InvalidMigratedFundsEntryAmount(amount) => {
                write!(f, "Invalid migrated funds entry amount: {}.", amount)
            }
            Error::InvalidOutputKind(k) => write!(f, "Invalid output kind: {}.", k),
            Error::InvalidParentsCount(count) => {
                write!(f, "Invalid parents count: {}.", count)
            }
            Error::InvalidPayloadKind(k) => write!(f, "Invalid payload kind: {}.", k),
            Error::InvalidPayloadLength(expected, actual) => {
                write!(f, "Invalid payload length: expected {}, got {}.", expected, actual)
            }
            Error::InvalidPowScoreValues(nps, npsmi) => write!(
                f,
                "Invalid pow score values: next pow score {} and next pow score milestone index {}.",
                nps, npsmi
            ),
            Error::InvalidReceiptFundsCount(count) => write!(f, "Invalid receipt funds count: {}.", count),
            Error::InvalidReferenceIndex(index) => write!(f, "Invalid reference index: {}.", index),
            Error::InvalidSignature => write!(f, "Invalid signature provided."),
            Error::InvalidSignatureKind(k) => write!(f, "Invalid signature kind: {}.", k),
            Error::InvalidTailTransactionHash => write!(f, "Invalid tail transaction hash."),
            Error::InvalidTokenSchemeKind(k) => write!(f, "Invalid token scheme kind {}.", k),
            Error::InvalidTreasuryAmount(amount) => write!(f, "Invalid treasury amount: {}.", amount),
            Error::InvalidUnlockBlockCount(count) => write!(f, "Invalid unlock block count: {}.", count),
            Error::InvalidUnlockBlockKind(k) => write!(f, "Invalid unlock block kind: {}.", k),
            Error::InvalidUnlockBlockReference(index) => {
                write!(f, "Invalid unlock block reference: {0}", index)
            }
            Error::InvalidUnlockBlockAlias(index) => {
                write!(f, "Invalid unlock block alias: {0}", index)
            }
            Error::InvalidUnlockBlockNft(index) => {
                write!(f, "Invalid unlock block nft: {0}", index)
            }
            Error::Io(e) => write!(f, "I/O error happened: {}.", e),
            Error::MigratedFundsNotSorted => {
                write!(f, "Migrated funds are not sorted.")
            }
            Error::MilestoneInvalidPublicKeyCount(count) => {
                write!(f, "Invalid milestone public key count: {}.", count)
            }
            Error::MilestoneInvalidSignatureCount(count) => {
                write!(f, "Invalid milestone signature count: {}.", count)
            }
            Error::MilestonePublicKeysNotUniqueSorted => {
                write!(f, "Milestone public keys are not unique and/or sorted.")
            }
            Error::MilestonePublicKeysSignaturesCountMismatch(kcount, scount) => {
                write!(
                    f,
                    "Milestone public keys and signatures count mismatch: {0} != {1}.",
                    kcount, scount
                )
            }
            Error::MissingField(s) => write!(f, "Missing required field: {}.", s),
            Error::MissingPayload => write!(f, "Missing payload."),
            Error::ParentsNotUniqueSorted => {
                write!(f, "Parents are not unique and/or sorted.")
            }
            Error::ReceiptFundsNotUniqueSorted => {
                write!(f, "Receipt funds are not unique and/or sorted.")
            }
            Error::RemainingBytesAfterMessage => {
                write!(f, "Remaining bytes after message.")
            }
            Error::SignaturePublicKeyMismatch(expected, actual) => {
                write!(
                    f,
                    "Signature public key mismatch: expected {0}, got {1}.",
                    expected, actual
                )
            }
            Error::TailTransactionHashNotUnique(previous, current) => {
                write!(
                    f,
                    "Tail transaction hash is not unique at indices: {0} and {1}.",
                    previous, current
                )
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<CryptoError> for Error {
    fn from(error: CryptoError) -> Self {
        Error::CryptoError(error)
    }
}
