// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use alloc::string::String;
use core::{convert::Infallible, fmt};

use crypto::Error as CryptoError;
use prefix_hex::Error as HexError;
use primitive_types::U256;

use crate::{
    input::UtxoInput,
    output::{
        feature_block::FeatureBlockCount, unlock_condition::UnlockConditionCount, AliasId, MetadataFeatureBlockLength,
        NativeTokenCount, NftId, OutputAmount, OutputIndex, StateMetadataLength, StorageDepositAmount,
        TagFeatureBlockLength, TreasuryOutputAmount,
    },
    parent::ParentCount,
    payload::{
        milestone::BinaryParametersLength, InputCount, MigratedFundsAmount, MilestoneMetadataLength,
        MilestoneOptionCount, OutputCount, ReceiptFundsCount, SignatureCount, TagLength, TaggedDataLength,
    },
    unlock_block::{UnlockBlockCount, UnlockBlockIndex},
};

/// Error occurring when creating/parsing/validating messages.
#[derive(Debug, PartialEq)]
#[allow(missing_docs)]
pub enum Error {
    CannotReplaceMissingField,
    ConsumedAmountOverflow,
    ConsumedNativeTokensAmountOverflow,
    CreatedAmountOverflow,
    CreatedNativeTokensAmountOverflow,
    CryptoError(CryptoError),
    DuplicateSignatureUnlockBlock(u16),
    DuplicateUtxo(UtxoInput),
    ExpirationUnlockConditionZero,
    FeatureBlocksNotUniqueSorted,
    InputUnlockBlockCountMismatch { input_count: usize, block_count: usize },
    InvalidAddress,
    InvalidAddressKind(u8),
    InvalidAliasIndex(<UnlockBlockIndex as TryFrom<u16>>::Error),
    InvalidControllerKind(u8),
    InvalidStorageDepositAmount(<StorageDepositAmount as TryFrom<u64>>::Error),
    // The above is used by `Packable` to denote out-of-range values. The following denotes the actual amount.
    InsufficientStorageDepositAmount { amount: u64, required: u64 },
    StorageDepositReturnExceedsOutputAmount { deposit: u64, amount: u64 },
    InsufficientStorageDepositReturnAmount { deposit: u64, required: u64 },
    InvalidBinaryParametersLength(<BinaryParametersLength as TryFrom<usize>>::Error),
    InvalidEssenceKind(u8),
    InvalidFeatureBlockCount(<FeatureBlockCount as TryFrom<usize>>::Error),
    InvalidFeatureBlockKind(u8),
    InvalidFoundryOutputSupply { minted: U256, melted: U256, max: U256 },
    HexError(HexError),
    InvalidInputKind(u8),
    InvalidInputCount(<InputCount as TryFrom<usize>>::Error),
    InvalidInputOutputIndex(<OutputIndex as TryFrom<u16>>::Error),
    InvalidMessageLength(usize),
    InvalidStateMetadataLength(<StateMetadataLength as TryFrom<usize>>::Error),
    InvalidMetadataFeatureBlockLength(<MetadataFeatureBlockLength as TryFrom<usize>>::Error),
    InvalidMilestoneMetadataLength(<MilestoneMetadataLength as TryFrom<usize>>::Error),
    InvalidMilestoneOptionCount(<MilestoneOptionCount as TryFrom<usize>>::Error),
    InvalidMilestoneOptionKind(u8),
    InvalidMigratedFundsEntryAmount(<MigratedFundsAmount as TryFrom<u64>>::Error),
    InvalidNativeTokenCount(<NativeTokenCount as TryFrom<usize>>::Error),
    InvalidNftIndex(<UnlockBlockIndex as TryFrom<u16>>::Error),
    InvalidOutputAmount(<OutputAmount as TryFrom<u64>>::Error),
    InvalidOutputCount(<OutputCount as TryFrom<usize>>::Error),
    InvalidOutputKind(u8),
    InvalidParentCount(<ParentCount as TryFrom<usize>>::Error),
    InvalidPayloadKind(u32),
    InvalidPayloadLength { expected: usize, actual: usize },
    InvalidReceiptFundsCount(<ReceiptFundsCount as TryFrom<usize>>::Error),
    InvalidReceiptFundsSum(u128),
    InvalidReferenceIndex(<UnlockBlockIndex as TryFrom<u16>>::Error),
    InvalidSignature,
    InvalidSignatureKind(u8),
    InvalidTaggedDataLength(<TaggedDataLength as TryFrom<usize>>::Error),
    InvalidTagFeatureBlockLength(<TagFeatureBlockLength as TryFrom<usize>>::Error),
    InvalidTagLength(<TagLength as TryFrom<usize>>::Error),
    InvalidTailTransactionHash,
    InvalidTokenSchemeKind(u8),
    InvalidTransactionAmountSum(u128),
    InvalidTransactionNativeTokensCount(u16),
    InvalidTreasuryOutputAmount(<TreasuryOutputAmount as TryFrom<u64>>::Error),
    InvalidUnlockBlockCount(<UnlockBlockCount as TryFrom<usize>>::Error),
    InvalidUnlockBlockKind(u8),
    InvalidUnlockBlockReference(u16),
    InvalidUnlockBlockAlias(u16),
    InvalidUnlockBlockNft(u16),
    InvalidUnlockConditionCount(<UnlockConditionCount as TryFrom<usize>>::Error),
    InvalidUnlockConditionKind(u8),
    MigratedFundsNotSorted,
    MilestoneInvalidSignatureCount(<SignatureCount as TryFrom<usize>>::Error),
    MilestonePublicKeysSignaturesCountMismatch { key_count: usize, sig_count: usize },
    MilestoneOptionsNotUniqueSorted,
    MilestoneSignaturesNotUniqueSorted,
    MissingAddressUnlockCondition,
    MissingGovernorUnlockCondition,
    MissingPayload,
    MissingRequiredSenderBlock,
    MissingStateControllerUnlockCondition,
    NativeTokensNotUniqueSorted,
    NativeTokensNullAmount,
    NativeTokensOverflow,
    NonZeroStateIndexOrFoundryCounter,
    ParentsNotUniqueSorted,
    ProtocolVersionMismatch { expected: u8, actual: u8 },
    ReceiptFundsNotUniqueSorted,
    RemainingBytesAfterMessage,
    SelfControlledAliasOutput(AliasId),
    SelfDepositNft(NftId),
    SignaturePublicKeyMismatch { expected: String, actual: String },
    StorageDepositReturnOverflow,
    TailTransactionHashNotUnique { previous: usize, current: usize },
    TimelockUnlockConditionZero,
    UnallowedFeatureBlock { index: usize, kind: u8 },
    UnallowedUnlockCondition { index: usize, kind: u8 },
    UnlockConditionsNotUniqueSorted,
    UnsupportedOutputKind(u8),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CannotReplaceMissingField => write!(f, "cannot replace missing field"),
            Error::ConsumedAmountOverflow => write!(f, "consumed amount overflow"),
            Error::ConsumedNativeTokensAmountOverflow => write!(f, "consumed native tokens amount overflow"),
            Error::CreatedAmountOverflow => write!(f, "created amount overflow"),
            Error::CreatedNativeTokensAmountOverflow => write!(f, "created native tokens amount overflow"),
            Error::CryptoError(e) => write!(f, "cryptographic error: {}", e),
            Error::DuplicateSignatureUnlockBlock(index) => {
                write!(f, "duplicate signature unlock block at index: {0}", index)
            }
            Error::DuplicateUtxo(utxo) => write!(f, "duplicate UTXO {:?} in inputs", utxo),
            Error::ExpirationUnlockConditionZero => {
                write!(
                    f,
                    "expiration unlock condition with milestone index and timestamp set to 0",
                )
            }
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
            Error::InvalidAddress => write!(f, "invalid address provided"),
            Error::InvalidAddressKind(k) => write!(f, "invalid address kind: {}", k),
            Error::InvalidAliasIndex(index) => write!(f, "invalid alias index: {}", index),
            Error::InvalidBinaryParametersLength(length) => {
                write!(f, "invalid binary parameters length {length}")
            }
            Error::InvalidControllerKind(k) => write!(f, "invalid controller kind: {}", k),
            Error::InvalidStorageDepositAmount(amount) => {
                write!(f, "invalid storage deposit amount: {}", amount)
            }
            Error::InsufficientStorageDepositAmount { amount, required } => {
                write!(
                    f,
                    "insufficient output amount for storage deposit: {amount} (should be at least {required})"
                )
            }
            Error::InsufficientStorageDepositReturnAmount { deposit, required } => {
                write!(
                    f,
                    "the return deposit ({deposit}) must be greater than the minimum storage deposit ({required})"
                )
            }
            Error::StorageDepositReturnExceedsOutputAmount { deposit, amount } => write!(
                f,
                "storage deposit return of {deposit} exceeds the original output amount of {amount}"
            ),
            Error::InvalidEssenceKind(k) => write!(f, "invalid essence kind: {}", k),
            Error::InvalidFeatureBlockCount(count) => write!(f, "invalid feature block count: {}", count),
            Error::InvalidFeatureBlockKind(k) => write!(f, "invalid feature block kind: {}", k),
            Error::InvalidFoundryOutputSupply { minted, melted, max } => write!(
                f,
                "invalid foundry output supply: minted {minted}, melted {melted} max {max}",
            ),
            Error::HexError(error) => write!(f, "hex error: {}", error),
            Error::InvalidInputKind(k) => write!(f, "invalid input kind: {}", k),
            Error::InvalidInputCount(count) => write!(f, "invalid input count: {}", count),
            Error::InvalidInputOutputIndex(index) => write!(f, "invalid input or output index: {}", index),
            Error::InvalidMessageLength(length) => write!(f, "invalid message length {}", length),
            Error::InvalidStateMetadataLength(length) => write!(f, "invalid state metadata length {}", length),
            Error::InvalidMetadataFeatureBlockLength(length) => {
                write!(f, "invalid metadata feature block length {length}")
            }
            Error::InvalidMilestoneMetadataLength(length) => {
                write!(f, "invalid milestone metadata length {length}")
            }
            Error::InvalidMilestoneOptionCount(count) => write!(f, "invalid milestone option count: {count}"),
            Error::InvalidMilestoneOptionKind(k) => write!(f, "invalid milestone option kind: {k}"),
            Error::InvalidMigratedFundsEntryAmount(amount) => {
                write!(f, "invalid migrated funds entry amount: {amount}")
            }
            Error::InvalidNativeTokenCount(count) => write!(f, "invalid native token count: {}", count),
            Error::InvalidNftIndex(index) => write!(f, "invalid nft index: {}", index),
            Error::InvalidOutputAmount(amount) => write!(f, "invalid output amount: {}", amount),
            Error::InvalidOutputCount(count) => write!(f, "invalid output count: {}", count),
            Error::InvalidOutputKind(k) => write!(f, "invalid output kind: {}", k),
            Error::InvalidParentCount(count) => {
                write!(f, "invalid parents count: {}", count)
            }
            Error::InvalidPayloadKind(k) => write!(f, "invalid payload kind: {}", k),
            Error::InvalidPayloadLength { expected, actual } => {
                write!(f, "invalid payload length: expected {}, got {}", expected, actual)
            }
            Error::InvalidReceiptFundsCount(count) => write!(f, "invalid receipt funds count: {}", count),
            Error::InvalidReceiptFundsSum(sum) => write!(f, "invalid receipt amount sum: {sum}"),
            Error::InvalidReferenceIndex(index) => write!(f, "invalid reference index: {}", index),
            Error::InvalidSignature => write!(f, "invalid signature provided"),
            Error::InvalidSignatureKind(k) => write!(f, "invalid signature kind: {}", k),
            Error::InvalidTaggedDataLength(length) => {
                write!(f, "invalid tagged data length {}", length)
            }
            Error::InvalidTagFeatureBlockLength(length) => {
                write!(f, "invalid tag feature block length {}", length)
            }
            Error::InvalidTagLength(length) => {
                write!(f, "invalid tag length {}", length)
            }
            Error::InvalidTailTransactionHash => write!(f, "invalid tail transaction hash"),
            Error::InvalidTokenSchemeKind(k) => write!(f, "invalid token scheme kind {}", k),
            Error::InvalidTransactionAmountSum(value) => write!(f, "invalid transaction amount sum: {}", value),
            Error::InvalidTransactionNativeTokensCount(count) => {
                write!(f, "invalid transaction native tokens count: {}", count)
            }
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
            Error::MilestoneInvalidSignatureCount(count) => {
                write!(f, "invalid milestone signature count: {}", count)
            }
            Error::MilestonePublicKeysSignaturesCountMismatch { key_count, sig_count } => {
                write!(
                    f,
                    "milestone public keys and signatures count mismatch: {0} != {1}",
                    key_count, sig_count
                )
            }
            Error::MilestoneOptionsNotUniqueSorted => {
                write!(f, "milestone options are not unique and/or sorted")
            }
            Error::MilestoneSignaturesNotUniqueSorted => {
                write!(f, "milestone signatures are not unique and/or sorted")
            }
            Error::MissingAddressUnlockCondition => write!(f, "missing address unlock condition"),
            Error::MissingGovernorUnlockCondition => write!(f, "missing governor unlock condition"),
            Error::MissingPayload => write!(f, "missing payload"),
            Error::MissingRequiredSenderBlock => write!(f, "missing required sender block"),
            Error::MissingStateControllerUnlockCondition => write!(f, "missing state controller unlock condition"),
            Error::NativeTokensNotUniqueSorted => write!(f, "native tokens are not unique and/or sorted"),
            Error::NativeTokensNullAmount => write!(f, "native tokens null amount"),
            Error::NativeTokensOverflow => write!(f, "native tokens overflow"),
            Error::NonZeroStateIndexOrFoundryCounter => {
                write!(f, "non zero state index or foundry counter while alias ID is all zero")
            }
            Error::ParentsNotUniqueSorted => {
                write!(f, "parents are not unique and/or sorted")
            }
            Error::ProtocolVersionMismatch { expected, actual } => {
                write!(f, "protocol version mismatch: expected {expected}, got {actual}")
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
            Error::StorageDepositReturnOverflow => {
                write!(f, "storage deposit return overflow",)
            }
            Error::TailTransactionHashNotUnique { previous, current } => {
                write!(
                    f,
                    "tail transaction hash is not unique at indices: {0} and {1}",
                    previous, current
                )
            }
            Error::TimelockUnlockConditionZero => {
                write!(
                    f,
                    "timelock unlock condition with milestone index and timestamp set to 0",
                )
            }
            Error::UnallowedFeatureBlock { index, kind } => {
                write!(f, "unallowed feature block at index {} with kind {}", index, kind)
            }
            Error::UnallowedUnlockCondition { index, kind } => {
                write!(f, "unallowed unlock condition at index {} with kind {}", index, kind)
            }
            Error::UnlockConditionsNotUniqueSorted => write!(f, "unlock conditions are not unique and/or sorted"),
            Error::UnsupportedOutputKind(k) => write!(f, "unsupported output kind: {k}"),
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

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use super::*;

    #[derive(Debug)]
    pub enum DtoError {
        InvalidField(&'static str),
        Message(Error),
    }

    impl fmt::Display for DtoError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                DtoError::InvalidField(field) => write!(f, "{field}"),
                DtoError::Message(error) => write!(f, "{error}"),
            }
        }
    }

    impl From<Error> for DtoError {
        fn from(error: Error) -> Self {
            DtoError::Message(error)
        }
    }

    #[cfg(feature = "std")]
    impl std::error::Error for DtoError {}
}
