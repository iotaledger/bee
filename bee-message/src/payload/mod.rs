// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides types and syntactic validations of payloads.

pub mod data;
pub mod drng;
pub mod fpc;
pub mod indexation;
pub mod salt_declaration;
pub mod transaction;

use crate::{MessageUnpackError, ValidationError};

use data::DataPayload;
use drng::{ApplicationMessagePayload, BeaconPayload, CollectiveBeaconPayload, DkgPayload};
use fpc::FpcPayload;
use indexation::IndexationPayload;
use salt_declaration::SaltDeclarationPayload;
use transaction::{TransactionPayload, TransactionUnpackError};

use bee_packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
};

use alloc::boxed::Box;
use core::{convert::Infallible, fmt};

/// Maximum length (in bytes) of a message payload, defined in the specification:
/// <https://github.com/iotaledger/IOTA-2.0-Research-Specifications/blob/main/2.3%20Standard%20Payloads%20Layout.md>.
pub const PAYLOAD_LENGTH_MAX: u32 = 65157;

/// Error encountered unpacking a payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum PayloadUnpackError {
    Transaction(TransactionUnpackError),
    InvalidKind(u32),
    Validation(ValidationError),
}

impl_wrapped_validated!(
    PayloadUnpackError,
    PayloadUnpackError::Transaction,
    TransactionUnpackError
);
impl_wrapped_variant!(PayloadUnpackError, PayloadUnpackError::Validation, ValidationError);
impl_from_infallible!(PayloadUnpackError);

impl fmt::Display for PayloadUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transaction(e) => write!(f, "error unpacking transaction payload: {}", e),
            Self::InvalidKind(kind) => write!(f, "invalid payload kind: {}.", kind),
            Self::Validation(e) => write!(f, "{}", e),
        }
    }
}

/// Common features and attributes of message payloads.
pub trait MessagePayload: Packable + Into<Payload> {
    /// Kind of the payload.
    const KIND: u32;
    /// Version of the payload.
    const VERSION: u8;

    /// Packs a payload, its type and version.
    fn pack_payload<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        Self::KIND.pack(packer)?;
        Self::VERSION.pack(packer)?;
        self.pack(packer)
    }

    /// Unpacks a payload, its type and version.
    fn unpack_payload<U: Unpacker, E, const VERIFY: bool>(unpacker: &mut U) -> Result<Payload, UnpackError<E, U::Error>>
    where
        E: From<MessageUnpackError> + From<ValidationError> + From<Self::UnpackError>,
    {
        let version = u8::unpack::<_, VERIFY>(unpacker).infallible()?;

        if version != Self::VERSION {
            return Err(ValidationError::InvalidPayloadVersion {
                version,
                payload_kind: Self::KIND,
            })
            .map_err(UnpackError::from_packable)?;
        }

        Ok(Self::unpack::<_, VERIFY>(unpacker).coerce()?.into())
    }
}

/// A generic payload that can represent different types defining message payloads.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Payload {
    /// A pure data payload.
    Data(Box<DataPayload>),
    /// A transaction payload.
    Transaction(Box<TransactionPayload>),
    /// An FPC payload.
    Fpc(Box<FpcPayload>),
    /// A dRNG application message payload.
    ApplicationMessage(Box<ApplicationMessagePayload>),
    /// A dRNG DKG payload.
    Dkg(Box<DkgPayload>),
    /// A dRNG beacon payload.
    Beacon(Box<BeaconPayload>),
    /// A dRNG collective beacon payload.
    CollectiveBeacon(Box<CollectiveBeaconPayload>),
    /// A salt declaration payload.
    SaltDeclaration(Box<SaltDeclarationPayload>),
    /// An indexation payload.
    Indexation(Box<IndexationPayload>),
}

impl Payload {
    /// Returns the payload kind of a [`Payload`].
    pub fn kind(&self) -> u32 {
        match self {
            Self::Data(_) => DataPayload::KIND,
            Self::Transaction(_) => TransactionPayload::KIND,
            Self::Fpc(_) => FpcPayload::KIND,
            Self::ApplicationMessage(_) => ApplicationMessagePayload::KIND,
            Self::Dkg(_) => DkgPayload::KIND,
            Self::Beacon(_) => BeaconPayload::KIND,
            Self::CollectiveBeacon(_) => CollectiveBeaconPayload::KIND,
            Self::SaltDeclaration(_) => SaltDeclarationPayload::KIND,
            Self::Indexation(_) => IndexationPayload::KIND,
        }
    }
}

impl Packable for Payload {
    type UnpackError = MessageUnpackError;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match self {
            Self::Data(p) => p.pack_payload(packer),
            Self::Transaction(p) => p.pack_payload(packer),
            Self::Fpc(p) => p.pack_payload(packer),
            Self::ApplicationMessage(p) => p.pack_payload(packer),
            Self::Dkg(p) => p.pack_payload(packer),
            Self::Beacon(p) => p.pack_payload(packer),
            Self::CollectiveBeacon(p) => p.pack_payload(packer),
            Self::SaltDeclaration(p) => p.pack_payload(packer),
            Self::Indexation(p) => p.pack_payload(packer),
        }
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        match u32::unpack::<_, VERIFY>(unpacker).infallible()? {
            DataPayload::KIND => DataPayload::unpack_payload::<_, _, VERIFY>(unpacker),
            TransactionPayload::KIND => TransactionPayload::unpack_payload::<_, _, VERIFY>(unpacker),
            FpcPayload::KIND => FpcPayload::unpack_payload::<_, _, VERIFY>(unpacker),
            ApplicationMessagePayload::KIND => ApplicationMessagePayload::unpack_payload::<_, _, VERIFY>(unpacker),
            DkgPayload::KIND => DkgPayload::unpack_payload::<_, _, VERIFY>(unpacker),
            BeaconPayload::KIND => BeaconPayload::unpack_payload::<_, _, VERIFY>(unpacker),
            CollectiveBeaconPayload::KIND => CollectiveBeaconPayload::unpack_payload::<_, _, VERIFY>(unpacker),
            SaltDeclarationPayload::KIND => SaltDeclarationPayload::unpack_payload::<_, _, VERIFY>(unpacker),
            IndexationPayload::KIND => IndexationPayload::unpack_payload::<_, _, VERIFY>(unpacker),
            k => Err(UnpackError::Packable(PayloadUnpackError::InvalidKind(k).into())),
        }
    }
}

impl From<DataPayload> for Payload {
    fn from(payload: DataPayload) -> Self {
        Self::Data(Box::new(payload))
    }
}

impl From<TransactionPayload> for Payload {
    fn from(payload: TransactionPayload) -> Self {
        Self::Transaction(Box::new(payload))
    }
}

impl From<FpcPayload> for Payload {
    fn from(payload: FpcPayload) -> Self {
        Self::Fpc(Box::new(payload))
    }
}

impl From<ApplicationMessagePayload> for Payload {
    fn from(payload: ApplicationMessagePayload) -> Self {
        Self::ApplicationMessage(Box::new(payload))
    }
}

impl From<DkgPayload> for Payload {
    fn from(payload: DkgPayload) -> Self {
        Self::Dkg(Box::new(payload))
    }
}

impl From<BeaconPayload> for Payload {
    fn from(payload: BeaconPayload) -> Self {
        Self::Beacon(Box::new(payload))
    }
}

impl From<CollectiveBeaconPayload> for Payload {
    fn from(payload: CollectiveBeaconPayload) -> Self {
        Self::CollectiveBeacon(Box::new(payload))
    }
}

impl From<SaltDeclarationPayload> for Payload {
    fn from(payload: SaltDeclarationPayload) -> Self {
        Self::SaltDeclaration(Box::new(payload))
    }
}

impl From<IndexationPayload> for Payload {
    fn from(payload: IndexationPayload) -> Self {
        Self::Indexation(Box::new(payload))
    }
}

/// Representation of an optional [`Payload`].
/// Essentially an `Option<Payload>` with a different [`Packable`] implementation, to conform to specs.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[allow(missing_docs)]
pub enum OptionalPayload {
    None,
    Some(Payload),
}

impl Packable for OptionalPayload {
    type UnpackError = MessageUnpackError;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match self {
            Self::None => 0u32.pack(packer),
            Self::Some(payload) => {
                (payload.packed_len() as u32).pack(packer)?;
                payload.pack(packer)
            }
        }
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let len = u32::unpack::<_, VERIFY>(unpacker).infallible()? as usize;

        if len > 0 {
            let payload = Payload::unpack::<_, VERIFY>(unpacker)?;
            let actual_len = payload.packed_len();

            if len != actual_len {
                Err(UnpackError::Packable(
                    ValidationError::PayloadLengthMismatch {
                        expected: len,
                        actual: actual_len,
                    }
                    .into(),
                ))
            } else {
                Ok(Self::Some(payload))
            }
        } else {
            Ok(Self::None)
        }
    }
}

impl From<Option<Payload>> for OptionalPayload {
    fn from(option: Option<Payload>) -> Self {
        match option {
            None => Self::None,
            Some(payload) => Self::Some(payload),
        }
    }
}
