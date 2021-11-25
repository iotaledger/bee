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
    DuplicateSignatureUnlockBlock(u16),
    DuplicateUtxo(UtxoInput),
    FeatureBlocksNotUniqueSorted,
    InputUnlockBlockCountMismatch(usize, usize),
    InvalidAccumulatedOutput(u128),
    InvalidAddress,
    InvalidAddressKind(u8),
    InvalidAmount(u64),
    InvalidDustDepositAmount(u64),
    InvalidEssenceKind(u8),
    InvalidFeatureBlockCount(usize),
    InvalidFeatureBlockKind(u8),
    InvalidHexadecimalChar(String),
    InvalidHexadecimalLength(usize, usize),
    InvalidIndexationDataLength(usize),
    InvalidIndexationIndexLength(usize),
    InvalidInputKind(u8),
    InvalidInputOutputCount(u16),
    InvalidInputOutputIndex(u16),
    InvalidMessageLength(usize),
    InvalidMetadataLength(usize),
    InvalidMigratedFundsEntryAmount(u64),
    InvalidNativeTokenCount(usize),
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
    MissingRequiredSenderBlock,
    NativeTokensNotUniqueSorted,
    ParentsNotUniqueSorted,
    ReceiptFundsNotUniqueSorted,
    RemainingBytesAfterMessage,
    SignaturePublicKeyMismatch(String, String),
    TailTransactionHashNotUnique(usize, usize),
    UnallowedFeatureBlock(usize, u8),
    TooManyFeatureBlocks(usize, usize),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CryptoError(e) => write!(f, "cryptographic error: {}.", e),

            Error::DuplicateSignatureUnlockBlock(index) => {
                write!(f, "duplicate signature unlock block at index: {0}", index)
            }
            Error::DuplicateUtxo(utxo) => write!(f, "duplicate UTXO {:?} in inputs.", utxo),
            Error::FeatureBlocksNotUniqueSorted => write!(f, "feature blocks are not unique and/or sorted."),
            Error::InputUnlockBlockCountMismatch(input, block) => {
                write!(
                    f,
                    "input count and unlock block count mismatch: {} != {}.",
                    input, block
                )
            }
            Error::InvalidAccumulatedOutput(value) => write!(f, "invalid accumulated output balance: {}.", value),
            Error::InvalidAddress => write!(f, "invalid address provided."),
            Error::InvalidAddressKind(k) => write!(f, "invalid address kind: {}.", k),
            Error::InvalidAmount(amount) => write!(f, "invalid amount: {}.", amount),
            Error::InvalidDustDepositAmount(amount) => {
                write!(f, "invalid dust deposit amount: {}.", amount)
            }
            Error::InvalidEssenceKind(k) => write!(f, "invalid essence kind: {}.", k),
            Error::InvalidFeatureBlockCount(count) => write!(f, "invalid feature block count: {}.", count),
            Error::InvalidFeatureBlockKind(k) => write!(f, "invalid feature block kind: {}.", k),
            Error::InvalidHexadecimalChar(hex) => write!(f, "invalid hexadecimal character: {}.", hex),
            Error::InvalidHexadecimalLength(expected, actual) => {
                write!(f, "invalid hexadecimal length: expected {} got {}.", expected, actual)
            }
            Error::InvalidIndexationDataLength(length) => {
                write!(f, "invalid indexation data length {}.", length)
            }
            Error::InvalidIndexationIndexLength(length) => {
                write!(f, "invalid indexation index length {}.", length)
            }
            Error::InvalidInputKind(k) => write!(f, "invalid input kind: {}.", k),
            Error::InvalidInputOutputCount(count) => write!(f, "invalid input or output count: {}.", count),
            Error::InvalidInputOutputIndex(index) => write!(f, "invalid input or output index: {}.", index),
            Error::InvalidMessageLength(length) => write!(f, "invalid message length {}.", length),
            Error::InvalidMetadataLength(length) => write!(f, "invalid metadata length {}.", length),
            Error::InvalidMigratedFundsEntryAmount(amount) => {
                write!(f, "invalid migrated funds entry amount: {}.", amount)
            }
            Error::InvalidNativeTokenCount(count) => write!(f, "invalid native token count: {}.", count),
            Error::InvalidOutputKind(k) => write!(f, "invalid output kind: {}.", k),
            Error::InvalidParentsCount(count) => {
                write!(f, "invalid parents count: {}.", count)
            }
            Error::InvalidPayloadKind(k) => write!(f, "invalid payload kind: {}.", k),
            Error::InvalidPayloadLength(expected, actual) => {
                write!(f, "invalid payload length: expected {}, got {}.", expected, actual)
            }
            Error::InvalidPowScoreValues(nps, npsmi) => write!(
                f,
                "invalid pow score values: next pow score {} and next pow score milestone index {}.",
                nps, npsmi
            ),
            Error::InvalidReceiptFundsCount(count) => write!(f, "invalid receipt funds count: {}.", count),
            Error::InvalidReferenceIndex(index) => write!(f, "invalid reference index: {}.", index),
            Error::InvalidSignature => write!(f, "invalid signature provided."),
            Error::InvalidSignatureKind(k) => write!(f, "invalid signature kind: {}.", k),
            Error::InvalidTailTransactionHash => write!(f, "invalid tail transaction hash."),
            Error::InvalidTokenSchemeKind(k) => write!(f, "invalid token scheme kind {}.", k),
            Error::InvalidTreasuryAmount(amount) => write!(f, "invalid treasury amount: {}.", amount),
            Error::InvalidUnlockBlockCount(count) => write!(f, "invalid unlock block count: {}.", count),
            Error::InvalidUnlockBlockKind(k) => write!(f, "invalid unlock block kind: {}.", k),
            Error::InvalidUnlockBlockReference(index) => {
                write!(f, "invalid unlock block reference: {0}", index)
            }
            Error::InvalidUnlockBlockAlias(index) => {
                write!(f, "invalid unlock block alias: {0}", index)
            }
            Error::InvalidUnlockBlockNft(index) => {
                write!(f, "invalid unlock block nft: {0}", index)
            }
            Error::Io(e) => write!(f, "i/o error happened: {}.", e),
            Error::MigratedFundsNotSorted => {
                write!(f, "migrated funds are not sorted.")
            }
            Error::MilestoneInvalidPublicKeyCount(count) => {
                write!(f, "invalid milestone public key count: {}.", count)
            }
            Error::MilestoneInvalidSignatureCount(count) => {
                write!(f, "invalid milestone signature count: {}.", count)
            }
            Error::MilestonePublicKeysNotUniqueSorted => {
                write!(f, "milestone public keys are not unique and/or sorted.")
            }
            Error::MilestonePublicKeysSignaturesCountMismatch(kcount, scount) => {
                write!(
                    f,
                    "milestone public keys and signatures count mismatch: {0} != {1}.",
                    kcount, scount
                )
            }
            Error::MissingField(s) => write!(f, "missing required field: {}.", s),
            Error::MissingPayload => write!(f, "missing payload."),
            Error::MissingRequiredSenderBlock => write!(f, "missing required sender block"),
            Error::NativeTokensNotUniqueSorted => write!(f, "native tokens are not unique and/or sorted."),
            Error::ParentsNotUniqueSorted => {
                write!(f, "parents are not unique and/or sorted.")
            }
            Error::ReceiptFundsNotUniqueSorted => {
                write!(f, "receipt funds are not unique and/or sorted.")
            }
            Error::RemainingBytesAfterMessage => {
                write!(f, "remaining bytes after message.")
            }
            Error::SignaturePublicKeyMismatch(expected, actual) => {
                write!(
                    f,
                    "signature public key mismatch: expected {0}, got {1}.",
                    expected, actual
                )
            }
            Error::TailTransactionHashNotUnique(previous, current) => {
                write!(
                    f,
                    "tail transaction hash is not unique at indices: {0} and {1}.",
                    previous, current
                )
            }
            Error::UnallowedFeatureBlock(index, kind) => {
                write!(f, "unallowed feature block at index {} with kind {}.", index, kind)
            }
            Error::TooManyFeatureBlocks(max, actual) => {
                write!(f, "too many feature blocks, max {}, got {}.", max, actual)
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
