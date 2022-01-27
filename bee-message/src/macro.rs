// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// TODO
#[macro_export]
macro_rules! impl_id {
    ($name:ident, $length:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(
            Clone,
            Copy,
            Eq,
            Hash,
            PartialEq,
            Ord,
            PartialOrd,
            derive_more::From,
            derive_more::AsRef,
            packable::Packable,
        )]
        #[as_ref(forward)]
        pub struct $name([u8; $name::LENGTH]);

        impl $name {
            #[doc = concat!("The length of a [`", stringify!($ty),"`].")]
            pub const LENGTH: usize = $length;

            #[doc = concat!("Creates a new [`", stringify!($ty),"`].")]
            pub fn new(bytes: [u8; $name::LENGTH]) -> Self {
                Self::from(bytes)
            }
        }

        impl core::str::FromStr for $name {
            type Err = crate::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok($name::from(crate::util::hex_decode(s)?))
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
                use alloc::string::ToString;

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

                    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

/// A convenience macro to work around the fact the `[bitflags]` crate does not yet support iterating over the
/// individual flags. This macro essentially creates the `[bitflags]` and puts the individual flags into an associated
/// constant `pub const ALL_FLAGS: &'static []`.
#[macro_export]
macro_rules! create_bitflags {
    ($(#[$meta:meta])* $Name:ident, $TagType:ty, [$(($FlagName:ident, $TypeName:ident),)+]) => {
        bitflags! {
            $(#[$meta])*
            pub struct $Name: $TagType {
                $(
                    #[doc = concat!("Signals the presence of a [`", stringify!($TypeName), "`].")]
                    const $FlagName = 1 << $TypeName::KIND;
                )*
            }
        }

        impl $Name {
            #[allow(dead_code)]
            /// Returns a slice of all possible base flags.
            pub const ALL_FLAGS: &'static [$Name] = &[$($Name::$FlagName),*];
        }
    };
}
