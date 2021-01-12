// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::boxed::Box;

const ED25519_PUBLIC_KEY_LENGTH: usize = 32;
const ED25519_SIGNATURE_LENGTH: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Ed25519Signature {
    public_key: [u8; ED25519_PUBLIC_KEY_LENGTH],
    // TODO size is 64, change with generic const.
    signature: Box<[u8]>,
}

impl Ed25519Signature {
    pub fn new(public_key: [u8; ED25519_PUBLIC_KEY_LENGTH], signature: Box<[u8]>) -> Self {
        Self { public_key, signature }
    }

    pub fn public_key(&self) -> &[u8; ED25519_PUBLIC_KEY_LENGTH] {
        &self.public_key
    }

    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
}

impl Packable for Ed25519Signature {
    type Error = Error;

    fn packed_len(&self) -> usize {
        ED25519_PUBLIC_KEY_LENGTH + ED25519_SIGNATURE_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        writer.write_all(&self.public_key)?;
        writer.write_all(&self.signature)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut public_key_bytes = [0u8; ED25519_PUBLIC_KEY_LENGTH];
        reader.read_exact(&mut public_key_bytes)?;

        let mut signature_bytes = vec![0u8; ED25519_SIGNATURE_LENGTH];
        reader.read_exact(&mut signature_bytes)?;

        Ok(Self {
            public_key: public_key_bytes,
            signature: signature_bytes.into_boxed_slice(),
        })
    }
}
