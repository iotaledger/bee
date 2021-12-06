// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::UnexpectedEOF, unpacker::Unpacker};

impl<'u> Unpacker for &'u [u8] {
    type Error = UnexpectedEOF;

    fn unpack_bytes<B: AsMut<[u8]>>(&mut self, mut bytes: B) -> Result<(), Self::Error> {
        let slice = bytes.as_mut();
        let len = slice.len();

        if self.len() >= len {
            let (head, tail) = self.split_at(len);
            *self = tail;
            slice.copy_from_slice(head);
            Ok(())
        } else {
            Err(UnexpectedEOF {
                required: len,
                had: self.len(),
            })
        }
    }

    #[inline]
    fn remaining_bytes(&self) -> Option<usize> {
        Some(self.len())
    }
}
