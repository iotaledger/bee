// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[macro_export]
macro_rules! string_serde_impl {
    ($type:ty) => {
        use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
        impl Serialize for $type {
            fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
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
                        let value = <$type>::from_str(v).map_err(|e| serde::de::Error::custom(e))?;
                        Ok(value)
                    }
                }
                deserializer.deserialize_str(StringVisitor)
            }
        }
    };
}
