// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod alias;
mod ed25519;
mod nft;

use alloc::{string::String, vec::Vec};

use bech32::{self, FromBase32, ToBase32, Variant};
use derive_more::From;
use packable::PackableExt;

pub use self::{alias::AliasAddress, ed25519::Ed25519Address, nft::NftAddress};
use crate::{
    output::{Output, OutputId},
    semantic::{ConflictReason, ValidationContext},
    signature::Signature,
    unlock_block::UnlockBlock,
    Error,
};

/// A generic address supporting different address kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, From, packable::Packable)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8, with_error = Error::InvalidAddressKind)]
#[packable(unpack_error = Error)]
pub enum Address {
    /// An Ed25519 address.
    #[packable(tag = Ed25519Address::KIND)]
    Ed25519(Ed25519Address),
    /// An alias address.
    #[packable(tag = AliasAddress::KIND)]
    Alias(AliasAddress),
    /// An NFT address.
    #[packable(tag = NftAddress::KIND)]
    Nft(NftAddress),
}

impl Address {
    /// Returns the address kind of an [`Address`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Ed25519(_) => Ed25519Address::KIND,
            Self::Alias(_) => AliasAddress::KIND,
            Self::Nft(_) => NftAddress::KIND,
        }
    }

    /// Checks whether the address is an [`Ed25519Address`].
    pub fn is_ed25519(&self) -> bool {
        matches!(self, Self::Ed25519(_))
    }

    /// Checks whether the address is an [`AliasAddress`].
    pub fn is_alias(&self) -> bool {
        matches!(self, Self::Alias(_))
    }

    /// Checks whether the address is an [`NftAddress`].
    pub fn is_nft(&self) -> bool {
        matches!(self, Self::Nft(_))
    }

    /// Tries to create an [`Address`] from a bech32 encoded string.
    pub fn try_from_bech32<T: AsRef<str>>(address: T) -> Result<(String, Self), Error> {
        match bech32::decode(address.as_ref()) {
            Ok((hrp, data, _)) => {
                let bytes = Vec::<u8>::from_base32(&data).map_err(|_| Error::InvalidAddress)?;
                Self::unpack_verified(&mut bytes.as_slice())
                    .map_err(|_| Error::InvalidAddress)
                    .map(|address| (hrp, address))
            }
            Err(_) => Err(Error::InvalidAddress),
        }
    }

    /// Encodes this address to a bech32 string with the given Human Readable Part as prefix.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_bech32<T: AsRef<str>>(&self, hrp: T) -> String {
        // PANIC: encoding can't fail as `self` has already been validated and built.
        bech32::encode(hrp.as_ref(), self.pack_to_vec().to_base32(), Variant::Bech32).unwrap()
    }

    ///
    pub fn unlock(
        &self,
        unlock_block: &UnlockBlock,
        inputs: &[(OutputId, &Output)],
        context: &mut ValidationContext,
    ) -> Result<(), ConflictReason> {
        match (self, unlock_block) {
            (Address::Ed25519(ed25519_address), UnlockBlock::Signature(unlock_block)) => {
                if context.unlocked_addresses.contains(self) {
                    return Err(ConflictReason::SemanticValidationFailed);
                }

                let Signature::Ed25519(signature) = unlock_block.signature();

                if signature.is_valid(&context.essence_hash, ed25519_address).is_err() {
                    return Err(ConflictReason::InvalidSignature);
                }

                context.unlocked_addresses.insert(*self);
            }
            (Address::Ed25519(_ed25519_address), UnlockBlock::Reference(_unlock_block)) => {
                // TODO actually check that it was unlocked by the same signature.
                if !context.unlocked_addresses.contains(self) {
                    return Err(ConflictReason::AddressNotUnlocked);
                }
            }
            (Address::Alias(alias_address), UnlockBlock::Alias(unlock_block)) => {
                // PANIC: indexing is fine as it is already syntactically verified that indexes reference below.
                if let (output_id, Output::Alias(alias_output)) = inputs[unlock_block.index() as usize] {
                    if &alias_output.alias_id().or_from_output_id(output_id) != alias_address.alias_id() {
                        return Err(ConflictReason::UnlockAddressMismatch);
                    }
                    if !context.unlocked_addresses.contains(self) {
                        return Err(ConflictReason::AddressNotUnlocked);
                    }
                } else {
                    return Err(ConflictReason::IncorrectUnlockMethod);
                }
            }
            (Address::Nft(nft_address), UnlockBlock::Nft(unlock_block)) => {
                // PANIC: indexing is fine as it is already syntactically verified that indexes reference below.
                if let (output_id, Output::Nft(nft_output)) = inputs[unlock_block.index() as usize] {
                    if &nft_output.nft_id().or_from_output_id(output_id) != nft_address.nft_id() {
                        return Err(ConflictReason::UnlockAddressMismatch);
                    }
                    if !context.unlocked_addresses.contains(self) {
                        return Err(ConflictReason::AddressNotUnlocked);
                    }
                } else {
                    return Err(ConflictReason::IncorrectUnlockMethod);
                }
            }
            _ => return Err(ConflictReason::IncorrectUnlockMethod),
        }

        Ok(())
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize, Serializer};
    use serde_json::Value;

    use super::*;
    pub use super::{alias::dto::AliasAddressDto, ed25519::dto::Ed25519AddressDto, nft::dto::NftAddressDto};
    use crate::error::dto::DtoError;

    /// Describes all the different address types.
    #[derive(Clone, Debug)]
    pub enum AddressDto {
        /// An Ed25519 address.
        Ed25519(Ed25519AddressDto),
        /// An alias address.
        Alias(AliasAddressDto),
        /// A NFT address.
        Nft(NftAddressDto),
    }

    impl From<&Address> for AddressDto {
        fn from(value: &Address) -> Self {
            match value {
                Address::Ed25519(a) => AddressDto::Ed25519(a.into()),
                Address::Alias(a) => AddressDto::Alias(a.into()),
                Address::Nft(a) => AddressDto::Nft(a.into()),
            }
        }
    }

    impl TryFrom<&AddressDto> for Address {
        type Error = DtoError;

        fn try_from(value: &AddressDto) -> Result<Self, Self::Error> {
            match value {
                AddressDto::Ed25519(a) => Ok(Address::Ed25519(a.try_into()?)),
                AddressDto::Alias(a) => Ok(Address::Alias(a.try_into()?)),
                AddressDto::Nft(a) => Ok(Address::Nft(a.try_into()?)),
            }
        }
    }

    impl<'de> Deserialize<'de> for AddressDto {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            let value = Value::deserialize(d)?;
            Ok(
                match value
                    .get("type")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| serde::de::Error::custom("invalid address type"))? as u8
                {
                    Ed25519Address::KIND => {
                        AddressDto::Ed25519(Ed25519AddressDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize ed25519 address: {}", e))
                        })?)
                    }
                    AliasAddress::KIND => {
                        AddressDto::Alias(AliasAddressDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize alias address: {}", e))
                        })?)
                    }
                    NftAddress::KIND => AddressDto::Nft(
                        NftAddressDto::deserialize(value)
                            .map_err(|e| serde::de::Error::custom(format!("cannot deserialize NFT address: {}", e)))?,
                    ),
                    _ => return Err(serde::de::Error::custom("invalid address type")),
                },
            )
        }
    }

    impl Serialize for AddressDto {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            #[derive(Serialize)]
            #[serde(untagged)]
            enum AddressDto_<'a> {
                T1(&'a Ed25519AddressDto),
                T2(&'a AliasAddressDto),
                T3(&'a NftAddressDto),
            }
            #[derive(Serialize)]
            struct TypedAddress<'a> {
                #[serde(flatten)]
                address: AddressDto_<'a>,
            }
            let address = match self {
                AddressDto::Ed25519(o) => TypedAddress {
                    address: AddressDto_::T1(o),
                },
                AddressDto::Alias(o) => TypedAddress {
                    address: AddressDto_::T2(o),
                },
                AddressDto::Nft(o) => TypedAddress {
                    address: AddressDto_::T3(o),
                },
            };
            address.serialize(serializer)
        }
    }
}
