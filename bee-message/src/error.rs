// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    input::UtxoInput,
    output::{
        feature_block::FeatureBlockCount, unlock_condition::UnlockConditionCount, AliasId, DustDepositAmount,
        ImmutableMetadataLength, MetadataFeatureBlockLength, NativeTokenCount, NftId, OutputIndex, StateMetadataLength,
        TagFeatureBlockLength, TreasuryOutputAmount,
    },
    parent::ParentCount,
    payload::{
        InputCount, MigratedFundsAmount, OutputCount, PublicKeyCount, ReceiptFundsCount, SignatureCount, TagLength,
        TaggedDataLength,
    },
    unlock_block::{AliasIndex, NftIndex, ReferenceIndex, UnlockBlockCount},
};

use crypto::Error as CryptoError;
use primitive_types::U256;

use alloc::string::String;
use core::{convert::Infallible, fmt};

/// Error occurring when creating/parsing/validating messages.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Error {
    CryptoError(CryptoError),
    DuplicateSignatureUnlockBlock(u16),
    DuplicateUtxo(UtxoInput),
    FeatureBlocksNotUniqueSorted,
    InputUnlockBlockCountMismatch { input_count: usize, block_count: usize },
    InvalidAccumulatedOutput(u128),
    InvalidAddress,
    InvalidAddressKind(u8),
    InvalidAliasIndex(<AliasIndex as TryFrom<u16>>::Error),
    InvalidControllerKind(u8),
    InvalidDustDepositAmount(<DustDepositAmount as TryFrom<u64>>::Error),
    InvalidEssenceKind(u8),
    InvalidFeatureBlockCount(<FeatureBlockCount as TryFrom<usize>>::Error),
    InvalidFeatureBlockKind(u8),
    InvalidFoundryOutputSupply { circulating: U256, max: U256 },
    InvalidHexadecimalChar(String),
    InvalidHexadecimalLength { expected: usize, actual: usize },
    InvalidTaggedDataLength(<TaggedDataLength as TryFrom<usize>>::Error),
    InvalidTagFeatureBlockLength(<TagFeatureBlockLength as TryFrom<usize>>::Error),
    InvalidTagLength(<TagLength as TryFrom<usize>>::Error),
    InvalidInputKind(u8),
    InvalidInputCount(<InputCount as TryFrom<usize>>::Error),
    InvalidOutputCount(<OutputCount as TryFrom<usize>>::Error),
    InvalidInputOutputIndex(<OutputIndex as TryFrom<u16>>::Error),
    InvalidMessageLength(usize),
    InvalidImmutableMetadataLength(<ImmutableMetadataLength as TryFrom<usize>>::Error),
    InvalidStateMetadataLength(<StateMetadataLength as TryFrom<usize>>::Error),
    InvalidMetadataFeatureBlockLength(<MetadataFeatureBlockLength as TryFrom<usize>>::Error),
    InvalidMigratedFundsEntryAmount(<MigratedFundsAmount as TryFrom<u64>>::Error),
    InvalidNativeTokenCount(<NativeTokenCount as TryFrom<usize>>::Error),
    InvalidNftIndex(<NftIndex as TryFrom<u16>>::Error),
    InvalidOutputKind(u8),
    InvalidParentCount(<ParentCount as TryFrom<usize>>::Error),
    InvalidPayloadKind(u32),
    InvalidPayloadLength { expected: usize, actual: usize },
    InvalidPowScoreValues { nps: u32, npsmi: u32 },
    InvalidReceiptFundsCount(<ReceiptFundsCount as TryFrom<usize>>::Error),
    InvalidReferenceIndex(<ReferenceIndex as TryFrom<u16>>::Error),
    InvalidSignature,
    InvalidSignatureKind(u8),
    InvalidTailTransactionHash,
    InvalidTokenSchemeKind(u8),
    InvalidTreasuryOutputAmount(<TreasuryOutputAmount as TryFrom<u64>>::Error),
    InvalidUnlockBlockCount(<UnlockBlockCount as TryFrom<usize>>::Error),
    InvalidUnlockBlockKind(u8),
    InvalidUnlockBlockReference(u16),
    InvalidUnlockBlockAlias(u16),
    InvalidUnlockBlockNft(u16),
    InvalidUnlockConditionCount(<UnlockConditionCount as TryFrom<usize>>::Error),
    InvalidUnlockConditionKind(u8),
    MigratedFundsNotSorted,
    MilestoneInvalidPublicKeyCount(<PublicKeyCount as TryFrom<usize>>::Error),
    MilestoneInvalidSignatureCount(<SignatureCount as TryFrom<usize>>::Error),
    MilestonePublicKeysNotUniqueSorted,
    MilestonePublicKeysSignaturesCountMismatch { key_count: usize, sig_count: usize },
    MissingField(&'static str),
    MissingPayload,
    MissingRequiredSenderBlock,
    NativeTokensNotUniqueSorted,
    NonZeroStateIndexOrFoundryCounter,
    ParentsNotUniqueSorted,
    ReceiptFundsNotUniqueSorted,
    RemainingBytesAfterMessage,
    SelfControlledAliasOutput(AliasId),
    SelfDepositNft(NftId),
    SignaturePublicKeyMismatch { expected: String, actual: String },
    TailTransactionHashNotUnique { previous: usize, current: usize },
    UnallowedFeatureBlock { index: usize, kind: u8 },
    UnallowedUnlockCondition { index: usize, kind: u8 },
    UnlockConditionsNotUniqueSorted,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CryptoError(e) => write!(f, "cryptographic error: {}", e),
            Error::DuplicateSignatureUnlockBlock(index) => {
                write!(f, "duplicate signature unlock block at index: {0}", index)
            }
            Error::DuplicateUtxo(utxo) => write!(f, "duplicate UTXO {:?} in inputs", utxo),
            Error::FeatureBlocksNotUniqueSorted => write!(f, "feature blocks are not unique and/or sorted"),
            Error::InputUnlockBlockCountMismatch {
                input_count,
                block_count,
            } => {
                write!(
                    f,
                    "input count and unlock block count mismatch: {} != {}",
                    input_count, block_count
                )
            }
            Error::InvalidAccumulatedOutput(value) => write!(f, "invalid accumulated output balance: {}", value),
            Error::InvalidAddress => write!(f, "invalid address provided"),
            Error::InvalidAddressKind(k) => write!(f, "invalid address kind: {}", k),
            Error::InvalidAliasIndex(index) => write!(f, "invalid alias index: {}", index),
            Error::InvalidControllerKind(k) => write!(f, "invalid controller kind: {}", k),
            Error::InvalidDustDepositAmount(amount) => {
                write!(f, "invalid dust deposit amount: {}", amount)
            }
            Error::InvalidEssenceKind(k) => write!(f, "invalid essence kind: {}", k),
            Error::InvalidFeatureBlockCount(count) => write!(f, "invalid feature block count: {}", count),
            Error::InvalidFeatureBlockKind(k) => write!(f, "invalid feature block kind: {}", k),
            Error::InvalidFoundryOutputSupply { circulating, max } => write!(
                f,
                "invalid foundry output supply: circulating {}, max {}",
                circulating, max
            ),
            Error::InvalidHexadecimalChar(hex) => write!(f, "invalid hexadecimal character: {}", hex),
            Error::InvalidHexadecimalLength { expected, actual } => {
                write!(f, "invalid hexadecimal length: expected {} got {}", expected, actual)
            }
            Error::InvalidTaggedDataLength(length) => {
                write!(f, "invalid tagged data length {}", length)
            }
            Error::InvalidTagFeatureBlockLength(length) => {
                write!(f, "invalid tag feature block length {}", length)
            }
            Error::InvalidTagLength(length) => {
                write!(f, "invalid tag length {}", length)
            }
            Error::InvalidInputKind(k) => write!(f, "invalid input kind: {}", k),
            Error::InvalidInputCount(count) => write!(f, "invalid input count: {}", count),
            Error::InvalidOutputCount(count) => write!(f, "invalid output count: {}", count),
            Error::InvalidInputOutputIndex(index) => write!(f, "invalid input or output index: {}", index),
            Error::InvalidMessageLength(length) => write!(f, "invalid message length {}", length),
            Error::InvalidStateMetadataLength(length) => write!(f, "invalid state metadata length {}", length),
            Error::InvalidImmutableMetadataLength(length) => write!(f, "invalid immutable metadata length {}", length),
            Error::InvalidMetadataFeatureBlockLength(length) => {
                write!(f, "invalid metadata feature block length {}", length)
            }
            Error::InvalidMigratedFundsEntryAmount(amount) => {
                write!(f, "invalid migrated funds entry amount: {}", amount)
            }
            Error::InvalidNativeTokenCount(count) => write!(f, "invalid native token count: {}", count),
            Error::InvalidNftIndex(index) => write!(f, "invalid nft index: {}", index),
            Error::InvalidOutputKind(k) => write!(f, "invalid output kind: {}", k),
            Error::InvalidParentCount(count) => {
                write!(f, "invalid parents count: {}", count)
            }
            Error::InvalidPayloadKind(k) => write!(f, "invalid payload kind: {}", k),
            Error::InvalidPayloadLength { expected, actual } => {
                write!(f, "invalid payload length: expected {}, got {}", expected, actual)
            }
            Error::InvalidPowScoreValues { nps, npsmi } => write!(
                f,
                "invalid pow score values: next pow score {} and next pow score milestone index {}",
                nps, npsmi
            ),
            Error::InvalidReceiptFundsCount(count) => write!(f, "invalid receipt funds count: {}", count),
            Error::InvalidReferenceIndex(index) => write!(f, "invalid reference index: {}", index),
            Error::InvalidSignature => write!(f, "invalid signature provided"),
            Error::InvalidSignatureKind(k) => write!(f, "invalid signature kind: {}", k),
            Error::InvalidTailTransactionHash => write!(f, "invalid tail transaction hash"),
            Error::InvalidTokenSchemeKind(k) => write!(f, "invalid token scheme kind {}", k),
            Error::InvalidTreasuryOutputAmount(amount) => write!(f, "invalid treasury amount: {}", amount),
            Error::InvalidUnlockBlockCount(count) => write!(f, "invalid unlock block count: {}", count),
            Error::InvalidUnlockBlockKind(k) => write!(f, "invalid unlock block kind: {}", k),
            Error::InvalidUnlockBlockReference(index) => {
                write!(f, "invalid unlock block reference: {0}", index)
            }
            Error::InvalidUnlockBlockAlias(index) => {
                write!(f, "invalid unlock block alias: {0}", index)
            }
            Error::InvalidUnlockBlockNft(index) => {
                write!(f, "invalid unlock block nft: {0}", index)
            }
            Error::InvalidUnlockConditionCount(count) => write!(f, "invalid unlock condition count: {}", count),
            Error::InvalidUnlockConditionKind(k) => write!(f, "invalid unlock condition kind: {}", k),
            Error::MigratedFundsNotSorted => {
                write!(f, "migrated funds are not sorted")
            }
            Error::MilestoneInvalidPublicKeyCount(count) => {
                write!(f, "invalid milestone public key count: {}", count)
            }
            Error::MilestoneInvalidSignatureCount(count) => {
                write!(f, "invalid milestone signature count: {}", count)
            }
            Error::MilestonePublicKeysNotUniqueSorted => {
                write!(f, "milestone public keys are not unique and/or sorted")
            }
            Error::MilestonePublicKeysSignaturesCountMismatch { key_count, sig_count } => {
                write!(
                    f,
                    "milestone public keys and signatures count mismatch: {0} != {1}",
                    key_count, sig_count
                )
            }
            Error::MissingField(s) => write!(f, "missing required field: {}", s),
            Error::MissingPayload => write!(f, "missing payload"),
            Error::MissingRequiredSenderBlock => write!(f, "missing required sender block"),
            Error::NativeTokensNotUniqueSorted => write!(f, "native tokens are not unique and/or sorted"),
            Error::NonZeroStateIndexOrFoundryCounter => {
                write!(f, "non zero state index or foundry counter while alias ID is all zero")
            }
            Error::ParentsNotUniqueSorted => {
                write!(f, "parents are not unique and/or sorted")
            }
            Error::ReceiptFundsNotUniqueSorted => {
                write!(f, "receipt funds are not unique and/or sorted")
            }
            Error::RemainingBytesAfterMessage => {
                write!(f, "remaining bytes after message")
            }
            Error::SelfControlledAliasOutput(alias_id) => {
                write!(f, "self controlled alias output, alias ID {}", alias_id)
            }
            Error::SelfDepositNft(nft_id) => {
                write!(f, "self deposit nft output, NFT ID {}", nft_id)
            }
            Error::SignaturePublicKeyMismatch { expected, actual } => {
                write!(
                    f,
                    "signature public key mismatch: expected {0}, got {1}",
                    expected, actual
                )
            }
            Error::TailTransactionHashNotUnique { previous, current } => {
                write!(
                    f,
                    "tail transaction hash is not unique at indices: {0} and {1}",
                    previous, current
                )
            }
            Error::UnallowedFeatureBlock { index, kind } => {
                write!(f, "unallowed feature block at index {} with kind {}", index, kind)
            }
            Error::UnallowedUnlockCondition { index, kind } => {
                write!(f, "unallowed unlock condition at index {} with kind {}", index, kind)
            }
            Error::UnlockConditionsNotUniqueSorted => write!(f, "unlock conditions are not unique and/or sorted"),
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
