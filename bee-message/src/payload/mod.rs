// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides types and syntactic validations of payloads.

pub mod data;
pub mod drng;
pub mod fpc;
pub mod indexation;
pub mod salt_declaration;
pub mod transaction;

use crate::{MessagePackError, MessageUnpackError, ValidationError};

use data::{DataPackError, DataPayload, DataUnpackError};
use drng::{
    ApplicationMessagePayload, BeaconPayload, CollectiveBeaconPayload, DkgPackError, DkgPayload, DkgUnpackError,
};
use fpc::{FpcPackError, FpcPayload, FpcUnpackError};
use indexation::{IndexationPackError, IndexationPayload, IndexationUnpackError};
use salt_declaration::{SaltDeclarationPackError, SaltDeclarationPayload, SaltDeclarationUnpackError};
use transaction::{TransactionPackError, TransactionPayload, TransactionUnpackError};

use bee_packable::{coerce::*, PackError, Packable, Packer, UnpackError, Unpacker};

use alloc::boxed::Box;
use core::{convert::Infallible, fmt};

/// Maximum length (in bytes) of a message payload, defined in the specification:
/// <https://github.com/iotaledger/IOTA-2.0-Research-Specifications/blob/main/2.3%20Standard%20Payloads%20Layout.md>.
pub const PAYLOAD_LENGTH_MAX: usize = 65157;

/// Error encountered packing a payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum PayloadPackError {
    Data(DataPackError),
    Transaction(TransactionPackError),
    Fpc(FpcPackError),
    Dkg(DkgPackError),
    SaltDeclaration(SaltDeclarationPackError),
    Indexation(IndexationPackError),
}

impl_wrapped_variant!(PayloadPackError, DataPackError, PayloadPackError::Data);
impl_wrapped_variant!(PayloadPackError, TransactionPackError, PayloadPackError::Transaction);
impl_wrapped_variant!(PayloadPackError, FpcPackError, PayloadPackError::Fpc);
impl_wrapped_variant!(PayloadPackError, DkgPackError, PayloadPackError::Dkg);
impl_wrapped_variant!(
    PayloadPackError,
    SaltDeclarationPackError,
    PayloadPackError::SaltDeclaration
);
impl_wrapped_variant!(PayloadPackError, IndexationPackError, PayloadPackError::Indexation);

impl_from_infallible!(PayloadPackError);

impl fmt::Display for PayloadPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Data(e) => write!(f, "error packing data payload: {}.", e),
            Self::Transaction(e) => write!(f, "error packing transaction payload: {}", e),
            Self::Fpc(e) => write!(f, "error packing FPC payload: {}.", e),
            Self::Dkg(e) => write!(f, "error packing DKG payload: {}", e),
            Self::SaltDeclaration(e) => write!(f, "error packing salt declaration payload: {}", e),
            Self::Indexation(e) => write!(f, "error packing indexation payload: {}", e),
        }
    }
}

/// Error encountered unpacking a payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum PayloadUnpackError {
    Data(DataUnpackError),
    Transaction(TransactionUnpackError),
    Fpc(FpcUnpackError),
    Dkg(DkgUnpackError),
    SaltDeclaration(SaltDeclarationUnpackError),
    Indexation(IndexationUnpackError),
    InvalidKind(u32),
    ValidationError(ValidationError),
}

impl_wrapped_variant!(PayloadUnpackError, DataUnpackError, PayloadUnpackError::Data);
impl_wrapped_variant!(
    PayloadUnpackError,
    TransactionUnpackError,
    PayloadUnpackError::Transaction
);
impl_wrapped_variant!(PayloadUnpackError, FpcUnpackError, PayloadUnpackError::Fpc);
impl_wrapped_variant!(PayloadUnpackError, DkgUnpackError, PayloadUnpackError::Dkg);
impl_wrapped_variant!(
    PayloadUnpackError,
    SaltDeclarationUnpackError,
    PayloadUnpackError::SaltDeclaration
);
impl_wrapped_variant!(
    PayloadUnpackError,
    IndexationUnpackError,
    PayloadUnpackError::Indexation
);
impl_wrapped_variant!(PayloadUnpackError, ValidationError, PayloadUnpackError::ValidationError);
impl_from_infallible!(PayloadUnpackError);

impl fmt::Display for PayloadUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Data(e) => write!(f, "error unpacking data payload: {}", e),
            Self::Transaction(e) => write!(f, "error unpacking transaction payload: {}", e),
            Self::Fpc(e) => write!(f, "error unpacking FPC payload: {}.", e),
            Self::Dkg(e) => write!(f, "error unpacking DKG payload: {}", e),
            Self::SaltDeclaration(e) => write!(f, "error unpacking salt declaration payload: {}", e),
            Self::Indexation(e) => write!(f, "error unpacking indexation payload: {}.", e),
            Self::InvalidKind(kind) => write!(f, "invalid Payload kind: {}.", kind),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// Common features and attributes of message payloads.
pub trait MessagePayload {
    /// Kind of the payload.
    const KIND: u32;
    /// Version of the payload.
    const VERSION: u8;
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
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        0u32.packed_len()
            + match self {
                Self::Data(p) => p.packed_len(),
                Self::Transaction(p) => p.packed_len(),
                Self::Fpc(p) => p.packed_len(),
                Self::ApplicationMessage(p) => p.packed_len(),
                Self::Dkg(p) => p.packed_len(),
                Self::Beacon(p) => p.packed_len(),
                Self::CollectiveBeacon(p) => p.packed_len(),
                Self::SaltDeclaration(p) => p.packed_len(),
                Self::Indexation(p) => p.packed_len(),
            }
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        match self {
            Self::Data(p) => {
                DataPayload::KIND.pack(packer).infallible()?;
                p.pack(packer)
            }
            Self::Transaction(p) => {
                TransactionPayload::KIND.pack(packer).infallible()?;
                p.pack(packer)
            }
            Self::Fpc(p) => {
                FpcPayload::KIND.pack(packer).infallible()?;
                p.pack(packer)
            }
            Self::ApplicationMessage(p) => {
                ApplicationMessagePayload::KIND.pack(packer).infallible()?;
                p.pack(packer)
            }
            Self::Dkg(p) => {
                DkgPayload::KIND.pack(packer).infallible()?;
                p.pack(packer)
            }
            Self::Beacon(p) => {
                BeaconPayload::KIND.pack(packer).infallible()?;
                p.pack(packer)
            }
            Self::CollectiveBeacon(p) => {
                CollectiveBeaconPayload::KIND.pack(packer).infallible()?;
                p.pack(packer)
            }
            Self::SaltDeclaration(p) => {
                SaltDeclarationPayload::KIND.pack(packer).infallible()?;
                p.pack(packer)
            }
            Self::Indexation(p) => {
                IndexationPayload::KIND.pack(packer).infallible()?;
                p.pack(packer)
            }
        }
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        match u32::unpack(unpacker).infallible()? {
            DataPayload::KIND => Ok(DataPayload::unpack(unpacker).coerce()?.into()),
            TransactionPayload::KIND => Ok(TransactionPayload::unpack(unpacker).coerce()?.into()),
            FpcPayload::KIND => Ok(FpcPayload::unpack(unpacker).coerce()?.into()),
            ApplicationMessagePayload::KIND => Ok(ApplicationMessagePayload::unpack(unpacker).coerce()?.into()),
            DkgPayload::KIND => Ok(DkgPayload::unpack(unpacker).coerce()?.into()),
            BeaconPayload::KIND => Ok(BeaconPayload::unpack(unpacker).coerce()?.into()),
            CollectiveBeaconPayload::KIND => Ok(CollectiveBeaconPayload::unpack(unpacker).coerce()?.into()),
            SaltDeclarationPayload::KIND => Ok(SaltDeclarationPayload::unpack(unpacker).coerce()?.into()),
            IndexationPayload::KIND => Ok(IndexationPayload::unpack(unpacker).coerce()?.into()),
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
