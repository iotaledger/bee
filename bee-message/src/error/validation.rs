// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, input::UtxoInput};

use crypto::Error as CryptoError;

use alloc::string::String;
use core::fmt;

#[derive(Debug)]
#[allow(missing_docs)]
pub enum ValidationError {
    CryptoError(CryptoError),
    DuplicateAddress(Address),
    DuplicateSignature(usize),
    DuplicateUtxo(UtxoInput),
    InputUnlockBlockCountMismatch(usize, usize),
    InvalidAccumulatedOutput(u128),
    InvalidAddress,
    InvalidAmount(u64),
    InvalidAssetBalanceLength(usize),
    InvalidEncryptedDealLength(usize),
    InvalidHexadecimalChar(String),
    InvalidHexadecimalLength(usize, usize),
    InvalidIndexationDataLength(usize),
    InvalidIndexationIndexLength(usize),
    InvalidInputCount(usize),
    InvalidMessageLength(usize),
    InvalidOutputCount(usize),
    InvalidOutputIndex(u16),
    InvalidParentsCount(usize),
    InvalidPayloadKind(u32),
    InvalidPayloadLength(usize),
    InvalidReferenceIndex(u16),
    InvalidSaltDeclarationBytesLength(usize),
    InvalidSignature,
    InvalidStrongParentsCount(usize),
    InvalidUnlockBlockCount(usize),
    InvalidUnlockBlockReference(usize),
    MissingField(&'static str),
    ParentsNotUniqueSorted,
    SignaturePublicKeyMismatch(String, String),
    TransactionInputsNotSorted,
    TransactionOutputsNotSorted,
}

impl_wrapped_variant!(ValidationError, CryptoError, ValidationError::CryptoError);

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CryptoError(e) => write!(f, "cryptographic error: {}", e),
            Self::DuplicateAddress(address) => write!(f, "duplicate address {:?} in outputs", address),
            Self::DuplicateSignature(index) => {
                write!(f, "duplicate signature at index: {}", index)
            }
            Self::DuplicateUtxo(utxo) => write!(f, "duplicate UTX {:?} in inputs", utxo),
            Self::InputUnlockBlockCountMismatch(input, block) => {
                write!(f, "input count and unlock block count mismatch: {} != {}", input, block,)
            }
            Self::InvalidAccumulatedOutput(value) => write!(f, "invalid accumulated output balance: {}", value),
            Self::InvalidAddress => write!(f, "invalid address provided"),
            Self::InvalidAssetBalanceLength(len) => {
                write!(f, "invalid asset allowance balance count: {}", len)
            }
            Self::InvalidAmount(amount) => write!(f, "invalid amount: {}", amount),
            Self::InvalidEncryptedDealLength(len) => write!(f, "invalid encrypted deal length: {}", len),
            Self::InvalidHexadecimalChar(hex) => write!(f, "invalid hexadecimal character: {}", hex),
            Self::InvalidHexadecimalLength(expected, actual) => {
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
            Self::InvalidOutputCount(count) => write!(f, "invalid output count: {}", count),
            Self::InvalidOutputIndex(index) => write!(f, "Inavlid output index: {}", index),
            Self::InvalidParentsCount(count) => write!(f, "invalid parents count: {}", count),
            Self::InvalidPayloadKind(kind) => write!(f, "invalid payload kind: {}", kind),
            Self::InvalidPayloadLength(len) => write!(f, "invalid payload length: {}", len),
            Self::InvalidReferenceIndex(index) => write!(f, "invalid reference index: {}", index),
            Self::InvalidSaltDeclarationBytesLength(len) => {
                write!(f, "invalid salt deeclaration bytes length: {}", len)
            }
            Self::InvalidSignature => write!(f, "invalid signature provided"),
            Self::InvalidStrongParentsCount(count) => write!(f, "invalid strong parents count: {}", count),
            Self::InvalidUnlockBlockCount(count) => write!(f, "invalid unlock block count: {}", count),
            Self::InvalidUnlockBlockReference(index) => {
                write!(f, "invalid unlock block reference: {}", index)
            }
            Self::MissingField(field) => write!(f, "missing required field: {}", field),
            Self::ParentsNotUniqueSorted => write!(f, "parents not unique and/or sorted"),
            Self::SignaturePublicKeyMismatch(expected, actual) => {
                write!(
                    f,
                    "signature public key mismatch: expected {}, got {}",
                    expected, actual,
                )
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
