// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    input::UtxoInput,
    output::{
        feature_block::FeatureBlockCount, unlock_condition::UnlockConditionCount, AliasId, MetadataFeatureBlockLength,
        NativeTokenCount, NftId, OutputAmount, OutputIndex, StateMetadataLength, StorageDepositAmount,
        TagFeatureBlockLength, TreasuryOutputAmount,
    },
    parent::ParentCount,
    payload::{
        InputCount, MigratedFundsAmount, OutputCount, PublicKeyCount, ReceiptFundsCount, SignatureCount, TagLength,
        TaggedDataLength,
    },
    unlock_block::{UnlockBlockCount, UnlockBlockIndex},
};
#[cfg(feature = "cpt2")]
use crate::{
    output::DustAllowanceAmount,
    payload::{IndexLength, IndexationDataLength},
};

use crypto::Error as CryptoError;
use primitive_types::U256;

use alloc::string::String;
use core::{convert::Infallible, fmt};

/// Error occurring when creating/parsing/validating messages.
#[derive(Debug, PartialEq)]
#[allow(missing_docs)]
pub enum Error {
    CryptoError(CryptoError),
    DuplicateSignatureUnlockBlock(u16),
    DuplicateUtxo(UtxoInput),
    ExpirationUnlockConditionZero,
    FeatureBlocksNotUniqueSorted,
    InputUnlockBlockCountMismatch {
        input_count: usize,
        block_count: usize,
    },
    InvalidAddress,
    InvalidAddressKind(u8),
    InvalidAliasIndex(<UnlockBlockIndex as TryFrom<u16>>::Error),
    InvalidControllerKind(u8),
    #[cfg(feature = "cpt2")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "cpt2")))]
    InvalidDustAllowanceAmount(<DustAllowanceAmount as TryFrom<u64>>::Error),
    InvalidStorageDepositAmount(<StorageDepositAmount as TryFrom<u64>>::Error),
    // The above is used by `Packable` to denote out-of-range values. The following denotes the actual amount.
    InsufficientStorageDepositAmount {
        amount: u64,
        required: u64,
    },
    StorageDepositReturnExceedsOutputAmount {
        deposit: u64,
        amount: u64,
    },
    InsufficientStorageDepositReturnAmount {
        deposit: u64,
        required: u64,
    },
    UnnecessaryStorageDepositReturnCondition {
        logical_amount: u64,
        required: u64,
    },
    InvalidEssenceKind(u8),
    InvalidFeatureBlockCount(<FeatureBlockCount as TryFrom<usize>>::Error),
    InvalidFeatureBlockKind(u8),
    InvalidFoundryOutputSupply {
        circulating: U256,
        max: U256,
    },
    HexInvalidPrefix {
        c0: char,
        c1: char,
    },
    HexInvalidHexCharacter {
        c: char,
        index: usize,
    },
    HexInvalidStringLength,
    HexInvalidStringLengthSlice {
        expected: usize,
        actual: usize,
    },
    HexOddLength,
    #[cfg(feature = "cpt2")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "cpt2")))]
    InvalidIndexationDataLength(<IndexationDataLength as TryFrom<usize>>::Error),
    #[cfg(feature = "cpt2")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "cpt2")))]
    InvalidIndexLength(<IndexLength as TryFrom<usize>>::Error),
    InvalidInputKind(u8),
    InvalidInputCount(<InputCount as TryFrom<usize>>::Error),
    InvalidInputOutputIndex(<OutputIndex as TryFrom<u16>>::Error),
    InvalidMessageLength(usize),
    InvalidStateMetadataLength(<StateMetadataLength as TryFrom<usize>>::Error),
    InvalidMetadataFeatureBlockLength(<MetadataFeatureBlockLength as TryFrom<usize>>::Error),
    InvalidMigratedFundsEntryAmount(<MigratedFundsAmount as TryFrom<u64>>::Error),
    InvalidNativeTokenCount(<NativeTokenCount as TryFrom<usize>>::Error),
    InvalidNftIndex(<UnlockBlockIndex as TryFrom<u16>>::Error),
    InvalidOutputAmount(<OutputAmount as TryFrom<u64>>::Error),
    InvalidOutputCount(<OutputCount as TryFrom<usize>>::Error),
    InvalidOutputKind(u8),
    InvalidParentCount(<ParentCount as TryFrom<usize>>::Error),
    InvalidPayloadKind(u32),
    InvalidPayloadLength {
        expected: usize,
        actual: usize,
    },
    InvalidPowScoreValues {
        nps: u32,
        npsmi: u32,
    },
    InvalidReceiptFundsCount(<ReceiptFundsCount as TryFrom<usize>>::Error),
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
    MilestoneInvalidPublicKeyCount(<PublicKeyCount as TryFrom<usize>>::Error),
    MilestoneInvalidSignatureCount(<SignatureCount as TryFrom<usize>>::Error),
    MilestonePublicKeysNotUniqueSorted,
    MilestonePublicKeysSignaturesCountMismatch {
        key_count: usize,
        sig_count: usize,
    },
    MissingAddressUnlockCondition,
    MissingField(&'static str),
    MissingGovernorUnlockCondition,
    MissingPayload,
    MissingRequiredSenderBlock,
    MissingStateControllerUnlockCondition,
    NativeTokensNotUniqueSorted,
    NativeTokensNullAmount,
    NonZeroStateIndexOrFoundryCounter,
    ParentsNotUniqueSorted,
    ReceiptFundsNotUniqueSorted,
    RemainingBytesAfterMessage,
    SelfControlledAliasOutput(AliasId),
    SelfDepositNft(NftId),
    SignaturePublicKeyMismatch {
        expected: String,
        actual: String,
    },
    TailTransactionHashNotUnique {
        previous: usize,
        current: usize,
    },
    TimelockUnlockConditionZero,
    UnallowedFeatureBlock {
        index: usize,
        kind: u8,
    },
    UnallowedUnlockCondition {
        index: usize,
        kind: u8,
    },
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
            Error::InvalidControllerKind(k) => write!(f, "invalid controller kind: {}", k),
            #[cfg(feature = "cpt2")]
            Error::InvalidDustAllowanceAmount(amount) => {
                write!(f, "invalid dust allowance amount: {}", amount)
            }
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
            Error::UnnecessaryStorageDepositReturnCondition {
                logical_amount,
                required,
            } => write!(
                f,
                "no storage deposit return is needed, the logical output amount {logical_amount} already covers the required deposit {required}"
            ),
            Error::InvalidEssenceKind(k) => write!(f, "invalid essence kind: {}", k),
            Error::InvalidFeatureBlockCount(count) => write!(f, "invalid feature block count: {}", count),
            Error::InvalidFeatureBlockKind(k) => write!(f, "invalid feature block kind: {}", k),
            Error::InvalidFoundryOutputSupply { circulating, max } => write!(
                f,
                "invalid foundry output supply: circulating {}, max {}",
                circulating, max
            ),
            Error::HexInvalidPrefix { c0, c1 } => {
                write!(f, "Invalid prefix `{c0}{c1}`, should be `0x`")
            }
            Error::HexInvalidHexCharacter { c, index } => {
                write!(f, "Invalid character {:?} at position {}", c, index)
            }
            Error::HexInvalidStringLength => write!(f, "Invalid string length"),
            Error::HexInvalidStringLengthSlice { expected, actual } => write!(
                f,
                "invalid hexadecimal length for slice: expected {expected} got {actual}"
            ),
            Error::HexOddLength => write!(f, "Odd number of digits in hex string"),
            #[cfg(feature = "cpt2")]
            Error::InvalidIndexationDataLength(length) => {
                write!(f, "invalid indexation data length {}", length)
            }
            #[cfg(feature = "cpt2")]
            Error::InvalidIndexLength(length) => {
                write!(f, "invalid index length {}", length)
            }
            Error::InvalidInputKind(k) => write!(f, "invalid input kind: {}", k),
            Error::InvalidInputCount(count) => write!(f, "invalid input count: {}", count),
            Error::InvalidInputOutputIndex(index) => write!(f, "invalid input or output index: {}", index),
            Error::InvalidMessageLength(length) => write!(f, "invalid message length {}", length),
            Error::InvalidStateMetadataLength(length) => write!(f, "invalid state metadata length {}", length),
            Error::InvalidMetadataFeatureBlockLength(length) => {
                write!(f, "invalid metadata feature block length {}", length)
            }
            Error::InvalidMigratedFundsEntryAmount(amount) => {
                write!(f, "invalid migrated funds entry amount: {}", amount)
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
            Error::InvalidPowScoreValues { nps, npsmi } => write!(
                f,
                "invalid pow score values: next pow score {} and next pow score milestone index {}",
                nps, npsmi
            ),
            Error::InvalidReceiptFundsCount(count) => write!(f, "invalid receipt funds count: {}", count),
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
            Error::MissingAddressUnlockCondition => write!(f, "missing address unlock condition"),
            Error::MissingField(s) => write!(f, "missing required field: {}", s),
            Error::MissingGovernorUnlockCondition => write!(f, "missing governor unlock condition"),
            Error::MissingPayload => write!(f, "missing payload"),
            Error::MissingRequiredSenderBlock => write!(f, "missing required sender block"),
            Error::MissingStateControllerUnlockCondition => write!(f, "missing state controller unlock condition"),
            Error::NativeTokensNotUniqueSorted => write!(f, "native tokens are not unique and/or sorted"),
            Error::NativeTokensNullAmount => write!(f, "native tokens null amount"),
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
