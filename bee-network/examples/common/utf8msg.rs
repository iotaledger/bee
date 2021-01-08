// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

#[derive(Clone)]
pub struct Utf8Message {
    message: String,
}

impl Utf8Message {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            message: String::from_utf8(bytes.to_vec()).unwrap(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        Vec::from(self.message.as_bytes())
    }
}

impl fmt::Display for Utf8Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
