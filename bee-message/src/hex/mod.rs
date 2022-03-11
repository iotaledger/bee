// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use hex::{FromHex, FromHexError};

use crate::Error;

use primitive_types::U256;

const PREFIX_LENGTH: usize = 2;

// Spec:

// Package hexutil implements hex encoding with 0x prefix. This encoding is used by the Ethereum RPC API to transport
// binary data in JSON payloads. Encoding Rules ¶

// All hex data must have prefix "0x".

// For byte slices, the hex data must be of even length. An empty byte slice encodes as "0x".

// Integers are encoded using the least amount of digits (no leading zero digits). Their encoding may be of uneven
// length. The number zero encodes as "0x0".

// Handling BigInts (that don't fit in json) (truncate)
impl Into<Error> for FromHexError {
    fn into(self) -> Error {
        match self {
            Self::InvalidHexCharacter { c, index } => Error::HexInvalidHexCharacter {
                c,
                index: index + PREFIX_LENGTH,
            },
            Self::InvalidStringLength => Error::HexInvalidStringLength,
            Self::OddLength => Error::HexOddLength,
        }
    }
}

/// Tries to decode an hexadecimal encoded string with a `0x` prefix.
pub trait FromHexPrefix: Sized {
    /// Tries to decode an hexadecimal encoded string with a `0x` prefix.
    fn from_hex_prefix(hex: &str) -> Result<Self, Error>;
}

// TODO: Maybe introduce `handle_error` with `#[cold]` attribute.
fn strip_prefix(hex: &str) -> Result<&str, Error> {
    if hex.starts_with("0x") {
        Ok(&hex[2..])
    } else if hex.len() < 2 {
        Err(Error::HexInvalidStringLength)
    } else {
        let mut chars = hex.chars();
        // Safety the following two operations are safe because we checked for the `hex.len()` in a previous branch.
        let c0 = chars.next().unwrap();
        let c1 = chars.next().unwrap();
        Err(Error::HexInvalidPrefix { c0, c1 })
    }
}

impl<const N: usize> FromHexPrefix for [u8; N]
where
    Self: FromHex,
{
    fn from_hex_prefix(hex: &str) -> Result<Self, Error> {
        let hex = strip_prefix(hex)?;
        let mut buffer = [0; N];
        hex::decode_to_slice(hex, &mut buffer).map_err(|e| match e {
            FromHexError::InvalidStringLength => Error::HexInvalidStringLengthSlice {
                expected: N * 2,
                actual: hex.len(),
            },
            _ => e.into(),
        })?;
        Ok(buffer)
    }
}

// TODO implement padding / odd length
impl FromHexPrefix for U256 {
    fn from_hex_prefix(hex: &str) -> Result<Self, Error> {
        let hex = strip_prefix(hex)?;
        let mut buffer = [0; std::mem::size_of::<U256>()];
        hex::decode_to_slice(hex, &mut buffer).map_err(|e| -> Error { e.into() })?;
        Ok(U256::from_little_endian(&buffer)) // TODO consider using `From` trait to avoid copy
    }
}

impl FromHexPrefix for Vec<u8> {
    fn from_hex_prefix(hex: &str) -> Result<Self, Error> {
        let hex = strip_prefix(hex)?;
        hex::decode(hex).map_err(|e| -> Error { e.into() })
    }
}

/// Encodes data into an hexadecimal encoded string with a `0x` prefix.
pub trait ToHexPrefix {
    /// Encodes data into an hexadecimal encoded string with a `0x` prefix.
    fn to_hex_prefix(self) -> String;
}

impl<T> ToHexPrefix for T
where
    T: AsRef<[u8]>,
{
    fn to_hex_prefix(self) -> String {
        format!("0x{}", hex::encode(self.as_ref()))
    }
}

///
pub fn hex_decode_prefix<T: FromHexPrefix>(hex: &str) -> Result<T, Error> {
    T::from_hex_prefix(hex)
}

///
pub fn hex_encode_prefix<T: AsRef<[u8]>>(value: T) -> String {
    ToHexPrefix::to_hex_prefix(value)
}
