// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::{Packable, Read, Write};

use bitflags::bitflags;
use serde::Serialize;

bitflags! {
    /// Flags representing the state of a message.
    #[derive(Default, Serialize)]
    pub struct Flags: u8 {
        /// The message is solid.
        const SOLID = 0b0000_0001;
        /// The message is a milestone.
        const MILESTONE = 0b0000_0010;
        /// The message has been referenced by a milestone.
        const REFERENCED = 0b0000_0100;
        /// The message is valid.
        const VALID = 0b0000_1000;
    }
}

impl Flags {
    /// Return whether the flags indicate that the message is solid.
    pub fn is_solid(&self) -> bool {
        self.contains(Flags::SOLID)
    }

    /// Set the solid flag for this message.
    pub fn set_solid(&mut self, is_solid: bool) {
        self.set(Flags::SOLID, is_solid);
    }

    /// Return whether the flags indicate that the message is a milestone.
    pub fn is_milestone(&self) -> bool {
        self.contains(Flags::MILESTONE)
    }

    /// Set the milestone flag for this message.
    pub fn set_milestone(&mut self, is_milestone: bool) {
        self.set(Flags::MILESTONE, is_milestone);
    }

    /// Return whether the flags indicate that the message is referenced.
    pub fn is_referenced(&self) -> bool {
        self.contains(Flags::REFERENCED)
    }

    /// Set the referenced flag for this message.
    pub fn set_referenced(&mut self, is_referenced: bool) {
        self.set(Flags::REFERENCED, is_referenced);
    }

    /// Return whether the flags indicate that the message is valid.
    pub fn is_valid(&self) -> bool {
        self.contains(Flags::VALID)
    }

    /// Set the valid flag for this message.
    pub fn set_valid(&mut self, is_valid: bool) {
        self.set(Flags::VALID, is_valid);
    }
}

impl Packable for Flags {
    type Error = std::io::Error;

    fn packed_len(&self) -> usize {
        self.bits().packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.bits().pack(writer)
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        // Flags are only expected to be unpacked from a trusted storage source.
        Ok(unsafe { Self::from_bits_unchecked(u8::unpack_inner::<R, CHECK>(reader)?) })
    }
}
