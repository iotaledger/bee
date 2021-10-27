// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    input::UtxoInput,
    output::PREFIXED_BALANCES_LENGTH_MAX,
    payload::{
        data::PREFIXED_DATA_LENGTH_MAX,
        drng::PREFIXED_LENGTH_MAX,
        fpc::{PREFIXED_CONFLICTS_LENGTH_MAX, PREFIXED_TIMESTAMPS_LENGTH_MAX},
        indexation::{
            PREFIXED_INDEXATION_DATA_LENGTH_MAX, PREFIXED_INDEXATION_INDEX_LENGTH_MAX,
            PREFIXED_INDEXATION_INDEX_LENGTH_MIN,
        },
        salt_declaration::PREFIXED_BYTES_LENGTH_MAX,
        transaction::{
            PREFIXED_INPUTS_LENGTH_MAX, PREFIXED_INPUTS_LENGTH_MIN, PREFIXED_OUTPUTS_LENGTH_MAX,
            PREFIXED_OUTPUTS_LENGTH_MIN,
        },
    },
    unlock::{PREFIXED_UNLOCK_BLOCKS_LENGTH_MAX, PREFIXED_UNLOCK_BLOCKS_LENGTH_MIN},
};

use bee_packable::{error::VecPrefixLengthError, InvalidBoundedU16, InvalidBoundedU32};
use crypto::Error as CryptoError;

use alloc::string::String;
use core::fmt;

#[derive(Debug)]
#[allow(missing_docs)]
pub enum ValidationError {
    AddressSignatureKindMismatch {
        expected: u8,
        actual: u8,
    },
    CryptoError(CryptoError),
    DuplicateAddress(Address),
    DuplicateSignature(usize),
    DuplicateUtxo(UtxoInput),
    InputUnlockBlockCountMismatch {
        inputs: usize,
        unlock_blocks: usize,
    },
    InvalidAccumulatedOutput(u128),
    InvalidAddress,
    InvalidAddressKind(u8),
    InvalidAmount(u64),
    InvalidAssetBalanceCount(VecPrefixLengthError<InvalidBoundedU32<0, PREFIXED_BALANCES_LENGTH_MAX>>),
    InvalidConflictsCount(VecPrefixLengthError<InvalidBoundedU32<0, PREFIXED_CONFLICTS_LENGTH_MAX>>),
    InvalidDataPayloadLength(VecPrefixLengthError<InvalidBoundedU32<0, PREFIXED_DATA_LENGTH_MAX>>),
    InvalidEncryptedDealLength(VecPrefixLengthError<InvalidBoundedU32<0, PREFIXED_LENGTH_MAX>>),
    InvalidHexadecimalChar(String),
    InvalidHexadecimalLength {
        expected: usize,
        actual: usize,
    },
    InvalidIndexationDataLength(VecPrefixLengthError<InvalidBoundedU32<0, PREFIXED_INDEXATION_DATA_LENGTH_MAX>>),
    InvalidIndexationIndexLength(
        VecPrefixLengthError<
            InvalidBoundedU32<PREFIXED_INDEXATION_INDEX_LENGTH_MIN, PREFIXED_INDEXATION_INDEX_LENGTH_MAX>,
        >,
    ),
    InvalidInputCount(VecPrefixLengthError<InvalidBoundedU32<PREFIXED_INPUTS_LENGTH_MIN, PREFIXED_INPUTS_LENGTH_MAX>>),
    InvalidMessageLength(usize),
    InvalidMessageVersion(u8),
    InvalidOutputCount(
        VecPrefixLengthError<InvalidBoundedU32<PREFIXED_OUTPUTS_LENGTH_MIN, PREFIXED_OUTPUTS_LENGTH_MAX>>,
    ),
    InvalidOutputIndex(u16),
    InvalidParentsBlocksCount(usize),
    InvalidParentsCount(usize),
    InvalidParentsKind(u8),
    InvalidPayloadKind(u32),
    InvalidPayloadVersion {
        version: u8,
        payload_kind: u32,
    },
    InvalidReferenceIndex(u16),
    InvalidSaltBytesLength(VecPrefixLengthError<InvalidBoundedU32<0, PREFIXED_BYTES_LENGTH_MAX>>),
    InvalidSignature,
    InvalidStrongParentsCount(usize),
    InvalidTimestampsCount(VecPrefixLengthError<InvalidBoundedU32<0, PREFIXED_TIMESTAMPS_LENGTH_MAX>>),
    InvalidUnlockBlockCount(
        VecPrefixLengthError<InvalidBoundedU16<PREFIXED_UNLOCK_BLOCKS_LENGTH_MIN, PREFIXED_UNLOCK_BLOCKS_LENGTH_MAX>>,
    ),
    InvalidUnlockBlockReference(usize),
    MissingBuilderField(&'static str),
    ParentsNotUniqueSorted,
    PayloadLengthMismatch {
        expected: usize,
        actual: usize,
    },
    SignaturePublicKeyMismatch {
        expected: String,
        actual: String,
    },
    TransactionInputsNotSorted,
    TransactionOutputsNotSorted,
}

impl_wrapped_variant!(ValidationError, ValidationError::CryptoError, CryptoError);

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddressSignatureKindMismatch { expected, actual } => {
                write!(
                    f,
                    "address and signature kind mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            Self::CryptoError(e) => write!(f, "cryptographic error: {}", e),
            Self::DuplicateAddress(address) => write!(f, "duplicate address {:?} in outputs", address),
            Self::DuplicateSignature(index) => {
                write!(f, "duplicate signature at index: {}", index)
            }
            Self::DuplicateUtxo(utxo) => write!(f, "duplicate UTX {:?} in inputs", utxo),
            Self::InputUnlockBlockCountMismatch { inputs, unlock_blocks } => {
                write!(
                    f,
                    "input count and unlock block count mismatch: {} != {}",
                    inputs, unlock_blocks
                )
            }
            Self::InvalidAccumulatedOutput(value) => write!(f, "invalid accumulated output balance: {}", value),
            Self::InvalidAddress => write!(f, "invalid address provided"),
            Self::InvalidAddressKind(kind) => write!(f, "invalid address kind: {}", kind),
            Self::InvalidAssetBalanceCount(len) => {
                write!(f, "invalid asset balance count: {}", len)
            }
            Self::InvalidAmount(amount) => write!(f, "invalid amount: {}", amount),
            Self::InvalidConflictsCount(count) => write!(f, "invalid conflicts count: {}", count),
            Self::InvalidDataPayloadLength(len) => write!(f, "invalid data payload length: {}", len),
            Self::InvalidEncryptedDealLength(len) => write!(f, "invalid encrypted deal length: {}", len),
            Self::InvalidHexadecimalChar(hex) => write!(f, "invalid hexadecimal character: {}", hex),
            Self::InvalidHexadecimalLength { expected, actual } => {
                write!(f, "invalid hexadecimal length: expected {} got {}", expected, actual)
            }
            Self::InvalidIndexationDataLength(len) => {
                write!(f, "invalid indexation data length: {}", len)
            }
            Self::InvalidIndexationIndexLength(len) => {
                write!(f, "invalid indexation index length: {}", len)
            }
            Self::InvalidInputCount(count) => write!(f, "invalid input count: {}", count),
            Self::InvalidMessageLength(len) => write!(f, "invalid message length: {}", len),
            Self::InvalidMessageVersion(version) => write!(f, "invalid message version: {}", version),
            Self::InvalidOutputCount(count) => write!(f, "invalid output count: {}", count),
            Self::InvalidOutputIndex(index) => write!(f, "invalid output index: {}", index),
            Self::InvalidParentsBlocksCount(count) => write!(f, "invalid parents blocks count: {}", count),
            Self::InvalidParentsCount(count) => write!(f, "invalid parents count: {}", count),
            Self::InvalidParentsKind(kind) => write!(f, "invalid parents kind: {}", kind),
            Self::InvalidPayloadKind(kind) => write!(f, "invalid payload kind: {}", kind),
            Self::InvalidPayloadVersion { version, payload_kind } => {
                write!(f, "invalid version {} for payload kind {}", version, payload_kind)
            }
            Self::InvalidReferenceIndex(index) => write!(f, "invalid reference index: {}", index),
            Self::InvalidSaltBytesLength(len) => {
                write!(f, "invalid salt deeclaration bytes length: {}", len)
            }
            Self::InvalidSignature => write!(f, "invalid signature provided"),
            Self::InvalidStrongParentsCount(count) => write!(f, "invalid strong parents count: {}", count),
            Self::InvalidTimestampsCount(count) => write!(f, "invalid timestamps count: {}", count),
            Self::InvalidUnlockBlockCount(count) => write!(f, "invalid unlock block count: {}", count),
            Self::InvalidUnlockBlockReference(index) => {
                write!(f, "invalid unlock block reference: {}", index)
            }
            Self::MissingBuilderField(field) => write!(f, "missing required builder field: {}", field),
            Self::ParentsNotUniqueSorted => write!(f, "parents not unique and/or sorted"),
            Self::SignaturePublicKeyMismatch { expected, actual } => {
                write!(
                    f,
                    "signature public key mismatch: expected {}, got {}",
                    expected, actual,
                )
            }
            Self::PayloadLengthMismatch { expected, actual } => {
                write!(f, "payload length mismatch: expected {} got {}", expected, actual)
            }
            Self::TransactionInputsNotSorted => {
                write!(f, "transaction inputs are not sorted")
            }
            Self::TransactionOutputsNotSorted => {
                write!(f, "transaction outputs are not sorted")
            }
        }
    }
}

#[cfg(std)]
impl std::error::Error for ValidationError {}
