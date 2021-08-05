// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use crate::{
    address::AddressUnpackError,
    input::InputUnpackError,
    output::{
        OutputIdUnpackError, OutputUnpackError, SignatureLockedAssetUnpackError, SignatureLockedSingleUnpackError,
    },
    payload::{
        data::DataUnpackError,
        drng::DkgUnpackError,
        fpc::FpcUnpackError,
        indexation::IndexationUnpackError,
        salt_declaration::SaltDeclarationUnpackError,
        transaction::{TransactionEssenceUnpackError, TransactionUnpackError},
        PayloadUnpackError,
    },
    signature::SignatureUnpackError,
    unlock::{UnlockBlockUnpackError, UnlockBlocksUnpackError},
    ValidationError,
};

use bee_packable::UnpackOptionError;

use core::{convert::Infallible, fmt};

/// Error encountered while deserializing with [`Packable`](bee_packable::Packable).
#[derive(Debug)]
#[allow(missing_docs)]
pub enum MessageUnpackError {
    Address(AddressUnpackError),
    Data(DataUnpackError),
    Dkg(DkgUnpackError),
    Fpc(FpcUnpackError),
    Indexation(IndexationUnpackError),
    Input(InputUnpackError),
    InvalidPayloadKind(u32),
    InvalidOptionTag(u8),
    Output(OutputUnpackError),
    OutputId(OutputIdUnpackError),
    Payload(PayloadUnpackError),
    SaltDeclaration(SaltDeclarationUnpackError),
    SignatureLockedAsset(SignatureLockedAssetUnpackError),
    SignatureLockedSingle(SignatureLockedSingleUnpackError),
    Signature(SignatureUnpackError),
    Transaction(TransactionUnpackError),
    TransactionEssence(TransactionEssenceUnpackError),
    UnlockBlock(UnlockBlockUnpackError),
    UnlockBlocks(UnlockBlocksUnpackError),
    ValidationError(ValidationError),
}

impl_wrapped_validated!(
    MessageUnpackError,
    IndexationUnpackError,
    MessageUnpackError::Indexation
);
impl_wrapped_validated!(MessageUnpackError, InputUnpackError, MessageUnpackError::Input);
impl_wrapped_validated!(MessageUnpackError, OutputUnpackError, MessageUnpackError::Output);
impl_wrapped_validated!(MessageUnpackError, PayloadUnpackError, MessageUnpackError::Payload);
impl_wrapped_validated!(
    MessageUnpackError,
    TransactionUnpackError,
    MessageUnpackError::Transaction
);
impl_wrapped_validated!(
    MessageUnpackError,
    TransactionEssenceUnpackError,
    MessageUnpackError::TransactionEssence
);
impl_wrapped_validated!(
    MessageUnpackError,
    SignatureLockedAssetUnpackError,
    MessageUnpackError::SignatureLockedAsset
);
impl_wrapped_validated!(
    MessageUnpackError,
    UnlockBlockUnpackError,
    MessageUnpackError::UnlockBlock
);
impl_wrapped_validated!(
    MessageUnpackError,
    UnlockBlocksUnpackError,
    MessageUnpackError::UnlockBlocks
);
impl_wrapped_variant!(MessageUnpackError, AddressUnpackError, MessageUnpackError::Address);
impl_wrapped_variant!(MessageUnpackError, DataUnpackError, MessageUnpackError::Data);
impl_wrapped_variant!(MessageUnpackError, DkgUnpackError, MessageUnpackError::Dkg);
impl_wrapped_variant!(MessageUnpackError, FpcUnpackError, MessageUnpackError::Fpc);
impl_wrapped_variant!(
    MessageUnpackError,
    SaltDeclarationUnpackError,
    MessageUnpackError::SaltDeclaration
);
impl_wrapped_variant!(MessageUnpackError, SignatureUnpackError, MessageUnpackError::Signature);
impl_wrapped_variant!(MessageUnpackError, ValidationError, MessageUnpackError::ValidationError);
impl_from_infallible!(MessageUnpackError);

impl From<SignatureLockedSingleUnpackError> for MessageUnpackError {
    fn from(error: SignatureLockedSingleUnpackError) -> Self {
        match error {
            SignatureLockedSingleUnpackError::ValidationError(e) => Self::ValidationError(e),
        }
    }
}

impl From<UnpackOptionError<MessageUnpackError>> for MessageUnpackError {
    fn from(error: UnpackOptionError<MessageUnpackError>) -> Self {
        match error {
            UnpackOptionError::Inner(error) => error,
            UnpackOptionError::UnknownTag(tag) => Self::InvalidOptionTag(tag),
        }
    }
}

impl fmt::Display for MessageUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Address(e) => write!(f, "error unpacking Address: {}", e),
            Self::Data(e) => write!(f, "error unpacking Data payload: {}", e),
            Self::Dkg(e) => write!(f, "error unpacking DKG payload: {}", e),
            Self::Fpc(e) => write!(f, "error unpacking FPC payload: {}", e),
            Self::Indexation(e) => write!(f, "error unpacking Indexation payload: {}", e),
            Self::Input(e) => write!(f, "error unpacking Input: {}", e),
            Self::InvalidPayloadKind(kind) => write!(f, "invalid payload kind: {}.", kind),
            Self::InvalidOptionTag(tag) => write!(f, "invalid tag for Option: {} is not 0 or 1", tag),
            Self::Output(e) => write!(f, "error unpacking Output: {}", e),
            Self::OutputId(e) => write!(f, "error unpacking OutputId: {}", e),
            Self::Payload(e) => write!(f, "error unpacking Payload: {}", e),
            Self::SaltDeclaration(e) => write!(f, "error unpacking SaltDeclaration payload: {}", e),
            Self::SignatureLockedAsset(e) => write!(f, "error unpacking SignatureLockedAsset: {}", e),
            Self::SignatureLockedSingle(e) => write!(f, "error unpacking SignatureLockedSingle: {}", e),
            Self::Signature(e) => write!(f, "error unpacking Signature: {}", e),
            Self::Transaction(e) => write!(f, "error unpacking Transaction payload: {}", e),
            Self::TransactionEssence(e) => write!(f, "error unpacking TransactionEssence: {}", e),
            Self::UnlockBlock(e) => write!(f, "error unpacking UnlockBlock: {}", e),
            Self::UnlockBlocks(e) => write!(f, "error unpacking UnlockBlocks: {}", e),
            Self::ValidationError(e) => write!(f, "validation error occured while unpacking: {}", e),
        }
    }
}
