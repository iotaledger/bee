// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use crate::{
    address::AddressUnpackError,
    input::InputUnpackError,
    output::{OutputIdUnpackError, OutputUnpackError},
    payload::{
        fpc::OpinionUnpackError,
        transaction::{TransactionEssenceUnpackError, TransactionUnpackError},
        PayloadUnpackError,
    },
    signature::SignatureUnpackError,
    unlock::UnlockBlockUnpackError,
    ValidationError,
};

use bee_packable::UnpackOptionError;

use core::{convert::Infallible, fmt};

/// Error encountered while deserializing with [`Packable`](bee_packable::Packable).
#[derive(Debug)]
#[allow(missing_docs)]
pub enum MessageUnpackError {
    Address(AddressUnpackError),
    Input(InputUnpackError),
    InvalidPayloadKind(u32),
    InvalidOptionTag(u8),
    Opinion(OpinionUnpackError),
    Output(OutputUnpackError),
    OutputId(OutputIdUnpackError),
    Payload(PayloadUnpackError),
    Signature(SignatureUnpackError),
    Transaction(TransactionUnpackError),
    TransactionEssence(TransactionEssenceUnpackError),
    UnlockBlock(UnlockBlockUnpackError),
    Validation(ValidationError),
}

impl_wrapped_validated!(MessageUnpackError, MessageUnpackError::Input, InputUnpackError);
impl_wrapped_validated!(MessageUnpackError, MessageUnpackError::Output, OutputUnpackError);
impl_wrapped_validated!(MessageUnpackError, MessageUnpackError::Payload, PayloadUnpackError);
impl_wrapped_validated!(
    MessageUnpackError,
    MessageUnpackError::Transaction,
    TransactionUnpackError
);
impl_wrapped_validated!(
    MessageUnpackError,
    MessageUnpackError::TransactionEssence,
    TransactionEssenceUnpackError
);
impl_wrapped_validated!(
    MessageUnpackError,
    MessageUnpackError::UnlockBlock,
    UnlockBlockUnpackError
);
impl_wrapped_variant!(MessageUnpackError, MessageUnpackError::Address, AddressUnpackError);
impl_wrapped_variant!(MessageUnpackError, MessageUnpackError::Opinion, OpinionUnpackError);
impl_wrapped_variant!(MessageUnpackError, MessageUnpackError::Signature, SignatureUnpackError);
impl_wrapped_variant!(MessageUnpackError, MessageUnpackError::Validation, ValidationError);
impl_from_infallible!(MessageUnpackError);

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
            Self::Input(e) => write!(f, "error unpacking Input: {}", e),
            Self::InvalidPayloadKind(kind) => write!(f, "invalid payload kind: {}.", kind),
            Self::InvalidOptionTag(tag) => write!(f, "invalid tag for Option: {} is not 0 or 1", tag),
            Self::Opinion(e) => write!(f, "error unpacking Opinion: {}", e),
            Self::Output(e) => write!(f, "error unpacking Output: {}", e),
            Self::OutputId(e) => write!(f, "error unpacking OutputId: {}", e),
            Self::Payload(e) => write!(f, "error unpacking Payload: {}", e),
            Self::Signature(e) => write!(f, "error unpacking Signature: {}", e),
            Self::Transaction(e) => write!(f, "error unpacking Transaction payload: {}", e),
            Self::TransactionEssence(e) => write!(f, "error unpacking TransactionEssence: {}", e),
            Self::UnlockBlock(e) => write!(f, "error unpacking UnlockBlock: {}", e),
            Self::Validation(e) => write!(f, "validation error occured while unpacking: {}", e),
        }
    }
}

#[cfg(std)]
impl std::error::Error for MessageUnpackError {}
