// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// TODO
#[macro_export]
macro_rules! impl_id {
    ($name:ident, $length:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, derive_more::From)]
        pub struct $name([u8; $name::LENGTH]);

        impl $name {
            #[doc = concat!("The length of a [`", stringify!($ty),"`].")]
            pub const LENGTH: usize = $length;

            #[doc = concat!("Creates a new [`", stringify!($ty),"`].")]
            pub fn new(bytes: [u8; $name::LENGTH]) -> Self {
                Self::from(bytes)
            }
        }

        #[cfg(feature = "serde1")]
        string_serde_impl!($name);

        impl core::str::FromStr for $name {
            type Err = crate::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let bytes: [u8; $name::LENGTH] = hex::decode(s)
                    .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
                    .try_into()
                    .map_err(|_| Self::Err::InvalidHexadecimalLength($name::LENGTH * 2, s.len()))?;

                Ok($name::from(bytes))
            }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "{}", hex::encode(self.0))
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "{}({})", stringify!($name), self)
            }
        }

        impl bee_common::packable::Packable for $name {
            type Error = crate::Error;

            fn packed_len(&self) -> usize {
                $name::LENGTH
            }

            fn pack<W: bee_common::packable::Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
                self.0.pack(writer)?;

                Ok(())
            }

            fn unpack_inner<R: bee_common::packable::Read + ?Sized, const CHECK: bool>(
                reader: &mut R,
            ) -> Result<Self, Self::Error> {
                Ok(Self::new(<[u8; $name::LENGTH]>::unpack_inner::<R, CHECK>(
                    reader,
                )?))
            }
        }
    };
}

/// Helper macro to serialize types to string via serde.
#[macro_export]
#[cfg(feature = "serde1")]
macro_rules! string_serde_impl {
    ($type:ty) => {
        use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

        impl Serialize for $type {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(&self.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<$type, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct StringVisitor;

                impl<'de> Visitor<'de> for StringVisitor {
                    type Value = $type;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        formatter.write_str("a string representing the value")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        let value = core::str::FromStr::from_str(v).map_err(serde::de::Error::custom)?;
                        Ok(value)
                    }
                }

                deserializer.deserialize_str(StringVisitor)
            }
        }
    };
}
