// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    constants::{INPUT_OUTPUT_INDEX_MAX, IOTA_SUPPLY, UNLOCK_BLOCK_COUNT_MAX, UNLOCK_BLOCK_COUNT_MIN},
    input::UtxoInput,
    output::DUST_THRESHOLD,
    parents::{MESSAGE_PARENTS_MAX, MESSAGE_PARENTS_MIN},
    payload::indexation::{INDEXATION_DATA_LENGTH_MAX, INDEXATION_INDEX_LENGTH_MAX, INDEXATION_INDEX_LENGTH_MIN},
};

use bee_packable::{
    bounded::{InvalidBoundedU16, InvalidBoundedU32, InvalidBoundedU64, InvalidBoundedU8},
    prefix::TryIntoPrefixError,
};

use crypto::Error as CryptoError;

use core::fmt;
use std::convert::Infallible;

/// Error occurring when creating/parsing/validating messages.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Error {
    CryptoError(CryptoError),
    DuplicateAddress(Address),
    DuplicateSignature(usize),
    DuplicateUtxo(UtxoInput),
    InputUnlockBlockCountMismatch(usize, usize),
    InvalidAccumulatedOutput(u128),
    InvalidAddress,
    InvalidAddressKind(u8),
    InvalidAmount(InvalidBoundedU64<1, IOTA_SUPPLY>),
    InvalidDustAllowanceAmount(InvalidBoundedU64<DUST_THRESHOLD, IOTA_SUPPLY>),
    InvalidEssenceKind(u8),
    InvalidHexadecimalChar(String),
    InvalidHexadecimalLength(usize, usize),
    InvalidIndexationDataLength(TryIntoPrefixError<InvalidBoundedU32<0, INDEXATION_DATA_LENGTH_MAX>>),
    InvalidIndexationIndexLength(
        TryIntoPrefixError<InvalidBoundedU16<INDEXATION_INDEX_LENGTH_MIN, INDEXATION_INDEX_LENGTH_MAX>>,
    ),
    InvalidInputKind(u8),
    InvalidInputOutputCount(usize),
    InvalidInputOutputIndex(InvalidBoundedU16<0, INPUT_OUTPUT_INDEX_MAX>),
    InvalidMessageLength(usize),
    InvalidMigratedFundsEntryAmount(u64),
    InvalidOutputKind(u8),
    InvalidParentsCount(TryIntoPrefixError<InvalidBoundedU8<MESSAGE_PARENTS_MIN, MESSAGE_PARENTS_MAX>>),
    InvalidPayloadKind(u32),
    InvalidPayloadLength(usize, usize),
    InvalidPowScoreValues(u32, u32),
    InvalidReceiptFundsCount(usize),
    InvalidReferenceIndex(InvalidBoundedU16<0, INPUT_OUTPUT_INDEX_MAX>),
    InvalidSignature,
    InvalidSignatureKind(u8),
    InvalidTailTransactionHash,
    InvalidTreasuryAmount(InvalidBoundedU64<0, IOTA_SUPPLY>),
    InvalidUnlockBlockCount(TryIntoPrefixError<InvalidBoundedU16<UNLOCK_BLOCK_COUNT_MIN, UNLOCK_BLOCK_COUNT_MAX>>),
    InvalidUnlockBlockKind(u8),
    InvalidUnlockBlockReference(usize),
    MigratedFundsNotSorted,
    MilestoneInvalidPublicKeyCount(usize),
    MilestoneInvalidSignatureCount(usize),
    MilestonePublicKeysNotUniqueSorted,
    MilestonePublicKeysSignaturesCountMismatch(usize, usize),
    MissingField(&'static str),
    MissingPayload,
    ParentsNotUniqueSorted,
    RemainingBytesAfterMessage,
    SignaturePublicKeyMismatch(String, String),
    TailTransactionHashNotUnique(usize, usize),
    TransactionInputsNotSorted,
    TransactionOutputsNotSorted,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CryptoError(e) => write!(f, "Cryptographic error: {}.", e),
            Error::DuplicateAddress(address) => write!(f, "Duplicate address {:?} in outputs of same kind.", address),
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
            Error::InvalidDustAllowanceAmount(amount) => write!(f, "Invalid dust allowance amount: {}.", amount),
            Error::InvalidEssenceKind(k) => write!(f, "Invalid essence kind: {}.", k),
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
            Error::InvalidTreasuryAmount(amount) => write!(f, "Invalid treasury amount: {}.", amount),
            Error::InvalidUnlockBlockCount(count) => write!(f, "Invalid unlock block count: {}.", count),
            Error::InvalidUnlockBlockKind(k) => write!(f, "Invalid unlock block kind: {}.", k),
            Error::InvalidUnlockBlockReference(index) => {
                write!(f, "Invalid unlock block reference: {0}", index)
            }
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
                write!(f, "Parents not unique and/or sorted.")
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
            Error::TransactionInputsNotSorted => {
                write!(f, "Transaction inputs are not sorted.")
            }
            Error::TransactionOutputsNotSorted => {
                write!(f, "Transaction outputs are not sorted.")
            }
        }
    }
}

impl From<CryptoError> for Error {
    fn from(error: CryptoError) -> Self {
        Error::CryptoError(error)
    }
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}
